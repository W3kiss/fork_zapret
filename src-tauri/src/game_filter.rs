//! Game Filter — mirrors `service.bat :game_switch_status` / `:save_game_filter_settings`.
//! State lives in `utils\game_filter.enabled` so it stays compatible with service.bat.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_RANGE: &str = "1024-65535";
/// Placeholder port service.bat substitutes when the filter is disabled.
const DISABLED_PORT: &str = "12";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameFilter {
    /// disabled | all | tcp | udp
    pub mode: String,
    pub tcp: String,
    pub udp: String,
}

impl Default for GameFilter {
    fn default() -> Self {
        Self { mode: "disabled".into(), tcp: DEFAULT_RANGE.into(), udp: DEFAULT_RANGE.into() }
    }
}

impl GameFilter {
    pub fn tcp_value(&self) -> String {
        match self.mode.as_str() {
            "all" | "tcp" => self.tcp.clone(),
            _ => DISABLED_PORT.into(),
        }
    }
    pub fn udp_value(&self) -> String {
        match self.mode.as_str() {
            "all" | "udp" => self.udp.clone(),
            _ => DISABLED_PORT.into(),
        }
    }
    /// `%GameFilter%` (legacy variable).
    pub fn combined_value(&self) -> String {
        match self.mode.as_str() {
            "all" | "tcp" => self.tcp.clone(),
            "udp" => self.udp.clone(),
            _ => DISABLED_PORT.into(),
        }
    }
}

fn flag_file(zapret: &Path) -> PathBuf {
    zapret.join("utils").join("game_filter.enabled")
}

pub fn load(zapret: &Path) -> GameFilter {
    let mut gf = GameFilter::default();
    let Ok(text) = crate::util::read_text(&flag_file(zapret)) else {
        return gf;
    };
    let (mut tcp_c, mut udp_c) = (None, None);
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (k, v) = match line.split_once('=') {
            Some((k, v)) => (k.trim().to_ascii_lowercase(), v.trim().to_string()),
            None => (line.to_ascii_lowercase(), String::new()),
        };
        match k.as_str() {
            "mode" => gf.mode = v.to_ascii_lowercase(),
            "all" => gf.mode = "all".into(),
            "udp" if v.is_empty() => gf.mode = "udp".into(),
            "udp" => udp_c = Some(v),
            "tcp" if v.is_empty() => gf.mode = "tcp".into(),
            "tcp" => tcp_c = Some(v),
            _ => {}
        }
    }
    if let Some(r) = tcp_c.and_then(|r| validate_range(&r)) {
        gf.tcp = r;
    }
    if let Some(r) = udp_c.and_then(|r| validate_range(&r)) {
        gf.udp = r;
    }
    if !matches!(gf.mode.as_str(), "all" | "tcp" | "udp") {
        gf.mode = "disabled".into();
    }
    gf
}

pub fn save(zapret: &Path, gf: &GameFilter) -> anyhow::Result<GameFilter> {
    let mode = match gf.mode.as_str() {
        m @ ("all" | "tcp" | "udp" | "disabled") => m.to_string(),
        _ => anyhow::bail!("invalid game filter mode"),
    };
    let tcp = validate_range(&gf.tcp).ok_or_else(|| anyhow::anyhow!("invalid TCP port range"))?;
    let udp = validate_range(&gf.udp).ok_or_else(|| anyhow::anyhow!("invalid UDP port range"))?;
    let file = flag_file(zapret);
    if let Some(d) = file.parent() {
        fs::create_dir_all(d)?;
    }
    fs::write(&file, format!("mode={mode}\r\ntcp={tcp}\r\nudp={udp}\r\n"))?;
    Ok(GameFilter { mode, tcp, udp })
}

/// Same rules as `:validate_game_filter_range`: comma-separated ports or ranges, 1..=65535.
pub fn validate_range(input: &str) -> Option<String> {
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.is_empty() {
        return None;
    }
    for item in cleaned.split(',') {
        let (a, b) = match item.split_once('-') {
            Some((a, b)) => (a, b),
            None => (item, item),
        };
        let ok = |s: &str| !s.is_empty() && s.len() <= 5 && !s.starts_with('0') && s.chars().all(|c| c.is_ascii_digit());
        if !ok(a) || !ok(b) {
            return None;
        }
        let (x, y): (u32, u32) = (a.parse().ok()?, b.parse().ok()?);
        if x > 65535 || y > 65535 || x > y {
            return None;
        }
    }
    Some(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ranges() {
        assert_eq!(validate_range("1024-65535"), Some("1024-65535".into()));
        assert_eq!(validate_range("1024-1934, 1936-65535"), Some("1024-1934,1936-65535".into()));
        assert!(validate_range("0-10").is_none());
        assert!(validate_range("100-50").is_none());
        assert!(validate_range("70000").is_none());
    }
}
