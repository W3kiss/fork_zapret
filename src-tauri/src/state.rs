//! Global application state managed by Tauri.

use crate::config::{self, AppConfig};
use crate::process::Engine;
use crate::autopick::{self, StrategyResult};
use crate::stats::StatsStore;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct AppPaths {
    pub data_dir: PathBuf,
    pub config_file: PathBuf,
    pub default_zapret: PathBuf,
    pub backups_dir: PathBuf,
    pub avatar_file: PathBuf,
    pub winws_log: PathBuf,
    pub stats_file: PathBuf,
    pub pick_file: PathBuf,
}

impl AppPaths {
    pub fn init(app: &AppHandle) -> anyhow::Result<Self> {
        let data_dir = app.path().app_local_data_dir()?;
        std::fs::create_dir_all(&data_dir)?;
        Ok(Self {
            config_file: data_dir.join("config.json"),
            default_zapret: data_dir.join("zapret"),
            backups_dir: data_dir.join("backups"),
            avatar_file: data_dir.join("avatar.png"),
            winws_log: data_dir.join("winws.log"),
            stats_file: data_dir.join("stats.json"),
            pick_file: data_dir.join("autopick.json"),
            data_dir,
        })
    }
}

pub struct AppState {
    pub paths: AppPaths,
    pub config: Mutex<AppConfig>,
    pub engine: Engine,
    /// Serialises connect/disconnect/update operations.
    pub op_lock: tokio::sync::Mutex<()>,
    pub autopick_running: AtomicBool,
    pub autopick_cancel: AtomicBool,
    pub updating: AtomicBool,
    /// Set when the user disconnected manually while Smart Mode is on,
    /// so the watcher does not immediately reconnect.
    pub smart_suppressed: AtomicBool,
    pub stats: StatsStore,
    /// Last auto-pick measurement per strategy file (availability + ping).
    pub pick_results: Mutex<HashMap<String, StrategyResult>>,
}

impl AppState {
    pub fn new(paths: AppPaths, cfg: AppConfig) -> Self {
        let stats = StatsStore::load(&paths.stats_file);
        let pick_results = Mutex::new(autopick::load_results(&paths.pick_file));
        Self {
            paths,
            config: Mutex::new(cfg),
            engine: Engine::default(),
            op_lock: tokio::sync::Mutex::new(()),
            autopick_running: AtomicBool::new(false),
            autopick_cancel: AtomicBool::new(false),
            updating: AtomicBool::new(false),
            smart_suppressed: AtomicBool::new(false),
            stats,
            pick_results,
        }
    }

    pub fn cfg(&self) -> AppConfig {
        self.config.lock().unwrap().clone()
    }

    pub fn update_cfg<F: FnOnce(&mut AppConfig)>(&self, f: F) -> AppConfig {
        let mut guard = self.config.lock().unwrap();
        f(&mut guard);
        let snapshot = guard.clone();
        drop(guard);
        if let Err(e) = config::save(&self.paths.config_file, &snapshot) {
            eprintln!("config save failed: {e}");
        }
        snapshot
    }

    /// Active zapret directory: custom path from config or the seeded copy in AppData.
    pub fn zapret_dir(&self) -> PathBuf {
        if let Some(p) = self.config.lock().unwrap().zapret_path.clone() {
            let p = PathBuf::from(p);
            if p.join("bin").join("winws.exe").exists() {
                return p;
            }
        }
        self.paths.default_zapret.clone()
    }
}

/// Copies the bundled zapret build into AppData on first launch.
pub fn seed_zapret(app: &AppHandle, target: &std::path::Path) -> anyhow::Result<()> {
    if target.join("bin").join("winws.exe").exists() {
        return Ok(());
    }
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(res) = app.path().resource_dir() {
        candidates.push(res.join("zapret"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("zapret"));
            candidates.push(dir.join("Zapret"));
        }
    }
    // Development fallback: <project>/../Zapret
    candidates.push(PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Zapret")));

    for c in candidates {
        if c.join("bin").join("winws.exe").exists() {
            crate::util::copy_dir_all(&c, target)?;
            return Ok(());
        }
    }
    anyhow::bail!("bundled zapret not found")
}
