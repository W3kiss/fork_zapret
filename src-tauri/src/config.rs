//! Persistent application configuration (JSON in the app data directory).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub fn default_smart_processes() -> Vec<String> {
    [
        "Discord.exe",
        "chrome.exe",
        "firefox.exe",
        "msedge.exe",
        "opera.exe",
        "brave.exe",
        "browser.exe", // Yandex Browser
        "yandex.exe",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfig {
    pub language: String,
    pub theme: String,
    pub onboarded: bool,
    pub strategy: String,
    pub autostart: bool,
    pub auto_connect: bool,
    pub minimize_to_tray: bool,
    pub smart_mode: bool,
    pub smart_processes: Vec<String>,
    pub hotkey: String,
    pub sounds: bool,
    pub zapret_path: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: "ru".into(),
            theme: "star".into(),
            onboarded: false,
            strategy: "general.bat".into(),
            autostart: false,
            auto_connect: false,
            minimize_to_tray: true,
            smart_mode: false,
            smart_processes: default_smart_processes(),
            hotkey: "Ctrl+Shift+Z".into(),
            sounds: true,
            zapret_path: None,
        }
    }
}

pub fn load(path: &Path) -> AppConfig {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(path: &Path, cfg: &AppConfig) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string_pretty(cfg)?)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
