//! Usage statistics: sessions, uptime, crashes, per-strategy time. Persisted to stats.json.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MAX_RECENT: usize = 30;

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub strategy: String,
    pub start: i64,
    pub seconds: u64,
    pub crashed: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Stats {
    pub sessions: u64,
    pub total_seconds: u64,
    pub crashes: u64,
    pub per_strategy: HashMap<String, u64>,
    pub last_connected: Option<i64>,
    pub recent: Vec<Session>,
}

pub struct StatsStore {
    stats: Mutex<Stats>,
    active: Mutex<Option<(String, i64)>>,
    path: PathBuf,
}

impl StatsStore {
    pub fn load(path: &Path) -> Self {
        let stats = fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Self { stats: Mutex::new(stats), active: Mutex::new(None), path: path.to_path_buf() }
    }

    fn persist(&self, stats: &Stats) {
        if let Ok(json) = serde_json::to_string_pretty(stats) {
            let _ = fs::write(&self.path, json);
        }
    }

    fn finalize(&self, crashed: bool) {
        let ended = self.active.lock().unwrap().take();
        if let Some((strategy, start)) = ended {
            let seconds = ((now_ms() - start).max(0) / 1000) as u64;
            let mut s = self.stats.lock().unwrap();
            s.total_seconds += seconds;
            *s.per_strategy.entry(strategy.clone()).or_default() += seconds;
            if crashed {
                s.crashes += 1;
            }
            s.recent.push(Session { strategy, start, seconds, crashed });
            if s.recent.len() > MAX_RECENT {
                let drop = s.recent.len() - MAX_RECENT;
                s.recent.drain(0..drop);
            }
            self.persist(&s);
        }
    }

    /// Starts a new session (finalising any previous one as a clean stop).
    pub fn begin(&self, strategy: &str) {
        self.finalize(false);
        let now = now_ms();
        {
            let mut s = self.stats.lock().unwrap();
            s.sessions += 1;
            s.last_connected = Some(now);
            self.persist(&s);
        }
        *self.active.lock().unwrap() = Some((strategy.to_string(), now));
    }

    pub fn end(&self, crashed: bool) {
        self.finalize(crashed);
    }

    pub fn snapshot(&self) -> Stats {
        self.stats.lock().unwrap().clone()
    }

    pub fn reset(&self) {
        *self.active.lock().unwrap() = None;
        let mut s = self.stats.lock().unwrap();
        *s = Stats::default();
        self.persist(&s);
    }
}
