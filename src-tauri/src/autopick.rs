//! Auto-pick: test every strategy, measure availability and latency, choose the best one.
//!
//! Ranking: more reachable targets first; among equally reliable strategies the lowest
//! median latency wins. Results are kept on disk so the strategy list can show ping.

use crate::controller::{self, publish};
use crate::process::StatusPayload;
use crate::state::AppState;
use crate::strategy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

pub const PROGRESS_EVENT: &str = "autopick://progress";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    /// testing | result | done | cancelled | failed
    pub phase: String,
    pub index: usize,
    pub total: usize,
    pub file: Option<String>,
    pub name: Option<String>,
    pub passed: usize,
    pub checks: usize,
    pub latency_ms: Option<u64>,
    pub found: Option<String>,
}

/// Last measured result for one strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyResult {
    pub file: String,
    pub passed: usize,
    pub checks: usize,
    /// Median latency of successful requests, ms.
    pub latency_ms: Option<u64>,
    /// Unix ms of the measurement.
    pub tested_at: i64,
}

pub fn load_results(path: &Path) -> HashMap<String, StrategyResult> {
    fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

fn save_results(path: &Path, r: &HashMap<String, StrategyResult>) {
    if let Ok(json) = serde_json::to_string_pretty(r) {
        let _ = fs::write(path, json);
    }
}

const FALLBACK_TARGETS: &[&str] = &[
    "https://discord.com",
    "https://gateway.discord.gg",
    "https://cdn.discordapp.com",
    "https://www.youtube.com",
    "https://i.ytimg.com",
    "https://redirector.googlevideo.com",
    "https://www.cloudflare.com",
];

/// Reads `utils\targets.txt` (format: `Key = "https://..."`), ignoring PING-only targets.
fn load_targets(zapret: &Path) -> Vec<String> {
    let text = crate::util::read_text(&zapret.join("utils").join("targets.txt")).unwrap_or_default();
    let mut out: Vec<String> = text
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split_once('=').map(|(_, v)| v.trim().trim_matches('"').to_string()))
        .filter(|v| v.starts_with("https://"))
        .collect();
    if out.is_empty() {
        out = FALLBACK_TARGETS.iter().map(|s| s.to_string()).collect();
    }
    out
}

/// Probes every target in parallel. Returns (reachable count, median latency of successes).
async fn probe(targets: &[String]) -> (usize, Option<u64>) {
    let client = match reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Fork-Zapret")
        .connect_timeout(Duration::from_secs(4))
        .timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .pool_max_idle_per_host(0)
        .build()
    {
        Ok(c) => c,
        Err(_) => return (0, None),
    };
    let futs = targets.iter().map(|url| {
        let c = client.clone();
        let url = url.clone();
        async move {
            let t0 = Instant::now();
            // Any HTTP response means the TLS handshake passed the DPI.
            match c.get(&url).send().await {
                Ok(_) => Some(t0.elapsed().as_millis() as u64),
                Err(_) => None,
            }
        }
    });
    let mut times: Vec<u64> = futures_util::future::join_all(futs).await.into_iter().flatten().collect();
    let passed = times.len();
    if times.is_empty() {
        return (0, None);
    }
    times.sort_unstable();
    (passed, Some(times[times.len() / 2]))
}

/// Sort key: higher pass count first, then lower latency (untested latency counts as worst).
fn rank_key(r: &StrategyResult) -> (std::cmp::Reverse<usize>, u64) {
    (std::cmp::Reverse(r.passed), r.latency_ms.unwrap_or(u64::MAX))
}

fn emit(app: &AppHandle, p: Progress) {
    let _ = app.emit(PROGRESS_EVENT, p);
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub async fn run(app: AppHandle) -> Result<Option<String>, String> {
    let st = app.state::<AppState>();
    if st.autopick_running.swap(true, Ordering::SeqCst) {
        return Err("AUTOPICK_RUNNING".into());
    }
    st.autopick_cancel.store(false, Ordering::SeqCst);
    let guard = st.op_lock.lock().await;

    let zapret = st.zapret_dir();
    let strategies = strategy::list(&zapret);
    let targets = load_targets(&zapret);
    let checks = targets.len();
    let total = strategies.len();
    let mut tested: Vec<StrategyResult> = Vec::new();
    let mut cancelled = false;

    for (i, s) in strategies.iter().enumerate() {
        if st.autopick_cancel.load(Ordering::SeqCst) {
            cancelled = true;
            break;
        }
        emit(
            &app,
            Progress {
                phase: "testing".into(),
                index: i + 1,
                total,
                file: Some(s.file.clone()),
                name: Some(s.name.clone()),
                passed: 0,
                checks,
                latency_ms: None,
                found: None,
            },
        );
        let (passed, latency_ms) = if controller::start_locked(&app, &s.file, "autopick").await.is_ok() {
            tokio::time::sleep(Duration::from_millis(800)).await;
            probe(&targets).await
        } else {
            (0, None)
        };
        let result = StrategyResult { file: s.file.clone(), passed, checks, latency_ms, tested_at: now_ms() };
        {
            let mut all = st.pick_results.lock().unwrap();
            all.insert(s.file.clone(), result.clone());
            save_results(&st.paths.pick_file, &all);
        }
        tested.push(result);
        emit(
            &app,
            Progress {
                phase: "result".into(),
                index: i + 1,
                total,
                file: Some(s.file.clone()),
                name: Some(s.name.clone()),
                passed,
                checks,
                latency_ms,
                found: None,
            },
        );
    }

    // Best = most reachable targets, then lowest ping; must reach at least half the targets.
    tested.sort_by_key(rank_key);
    let chosen = tested.first().filter(|r| r.passed > 0 && r.passed * 2 >= checks).cloned();

    let result = if cancelled {
        controller::stop_locked(&app, "autopick");
        emit(
            &app,
            Progress { phase: "cancelled".into(), index: 0, total, file: None, name: None, passed: 0, checks: 0, latency_ms: None, found: None },
        );
        Ok(None)
    } else if let Some(best) = chosen {
        let file = best.file.clone();
        st.update_cfg(|c| c.strategy = file.clone());
        // Leave the winner connected.
        let _ = controller::start_locked(&app, &file, "autopick").await;
        emit(
            &app,
            Progress {
                phase: "done".into(),
                index: total,
                total,
                name: Some(strategy::display_name(&file)),
                file: Some(file.clone()),
                passed: best.passed,
                checks,
                latency_ms: best.latency_ms,
                found: Some(file.clone()),
            },
        );
        Ok(Some(file))
    } else {
        controller::stop_locked(&app, "autopick");
        publish(&app, StatusPayload::default());
        emit(
            &app,
            Progress { phase: "failed".into(), index: total, total, file: None, name: None, passed: 0, checks, latency_ms: None, found: None },
        );
        Ok(None)
    };

    drop(guard);
    st.autopick_running.store(false, Ordering::SeqCst);
    result
}
