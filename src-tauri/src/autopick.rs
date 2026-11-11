//! Auto-find a working strategy: launch each ALT, probe blocked endpoints, keep the first that passes.

use crate::controller::{self, publish};
use crate::process::StatusPayload;
use crate::state::AppState;
use crate::strategy;
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::time::Duration;
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
    pub found: Option<String>,
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
fn load_targets(zapret: &std::path::Path) -> Vec<String> {
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

async fn probe(targets: &[String]) -> usize {
    let client = match reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Fork-Zapret")
        .connect_timeout(Duration::from_secs(4))
        .timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .pool_max_idle_per_host(0)
        .build()
    {
        Ok(c) => c,
        Err(_) => return 0,
    };
    let futs = targets.iter().map(|url| {
        let c = client.clone();
        let url = url.clone();
        async move {
            // Any HTTP response means the TLS handshake passed the DPI.
            c.get(&url).send().await.is_ok()
        }
    });
    futures_util::future::join_all(futs).await.into_iter().filter(|ok| *ok).count()
}

fn emit(app: &AppHandle, p: Progress) {
    let _ = app.emit(PROGRESS_EVENT, p);
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
    let total = strategies.len();
    let mut best: Option<(String, usize)> = None;
    let mut found: Option<String> = None;
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
                checks: targets.len(),
                found: None,
            },
        );
        if controller::start_locked(&app, &s.file, "autopick").await.is_err() {
            continue;
        }
        tokio::time::sleep(Duration::from_millis(800)).await;
        let passed = probe(&targets).await;
        emit(
            &app,
            Progress {
                phase: "result".into(),
                index: i + 1,
                total,
                file: Some(s.file.clone()),
                name: Some(s.name.clone()),
                passed,
                checks: targets.len(),
                found: None,
            },
        );
        if passed == targets.len() {
            found = Some(s.file.clone());
            break;
        }
        if passed > best.as_ref().map(|b| b.1).unwrap_or(0) {
            best = Some((s.file.clone(), passed));
        }
    }

    // Prefer a full pass; otherwise the best partial result (at least half of the checks).
    let chosen = found.clone().or_else(|| {
        best.filter(|(_, p)| *p * 2 >= targets.len()).map(|(f, _)| f)
    });

    let result = if cancelled {
        controller::stop_locked(&app, "autopick");
        emit(&app, Progress { phase: "cancelled".into(), index: 0, total, file: None, name: None, passed: 0, checks: 0, found: None });
        Ok(None)
    } else if let Some(file) = chosen {
        st.update_cfg(|c| c.strategy = file.clone());
        // Leave the working strategy connected (restart only if the last tested one differs).
        if found.is_none() || !st.engine.is_running() {
            let _ = controller::start_locked(&app, &file, "autopick").await;
        }
        emit(
            &app,
            Progress {
                phase: "done".into(),
                index: total,
                total,
                name: Some(strategy::display_name(&file)),
                file: Some(file.clone()),
                passed: 0,
                checks: targets.len(),
                found: Some(file.clone()),
            },
        );
        Ok(Some(file))
    } else {
        controller::stop_locked(&app, "autopick");
        publish(&app, StatusPayload::default());
        emit(&app, Progress { phase: "failed".into(), index: total, total, file: None, name: None, passed: 0, checks: targets.len(), found: None });
        Ok(None)
    };

    drop(guard);
    st.autopick_running.store(false, Ordering::SeqCst);
    result
}
