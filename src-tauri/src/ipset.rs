//! IPSet modes — mirrors `service.bat :ipset_switch_status` / `:ipset_switch`.
//!
//! * `any`    — `ipset-all.txt` is empty (matches every IP)
//! * `none`   — `ipset-all.txt` contains only the placeholder `203.0.113.113/32`
//! * `loaded` — full list; switching away stores it in `ipset-all.txt.backup`

use std::fs;
use std::path::{Path, PathBuf};

pub const PLACEHOLDER: &str = "203.0.113.113/32";
const IPSET_URL: &str =
    "https://raw.githubusercontent.com/Flowseal/zapret-discord-youtube/refs/heads/main/.service/ipset-service.txt";

fn list_file(z: &Path) -> PathBuf {
    z.join("lists").join("ipset-all.txt")
}
fn backup_file(z: &Path) -> PathBuf {
    z.join("lists").join("ipset-all.txt.backup")
}

pub fn status(z: &Path) -> String {
    let text = crate::util::read_text(&list_file(z)).unwrap_or_default();
    if text.trim().is_empty() {
        "any".into()
    } else if text.lines().any(|l| l.trim() == PLACEHOLDER) {
        "none".into()
    } else {
        "loaded".into()
    }
}

pub fn has_backup(z: &Path) -> bool {
    backup_file(z).exists()
}

fn stash_loaded(z: &Path) -> std::io::Result<()> {
    let (list, backup) = (list_file(z), backup_file(z));
    if backup.exists() {
        fs::remove_file(&backup)?;
    }
    fs::rename(&list, &backup)
}

pub fn set_mode(z: &Path, target: &str) -> anyhow::Result<String> {
    let current = status(z);
    if current == target {
        return Ok(current);
    }
    let list = list_file(z);
    match target {
        "none" => {
            if current == "loaded" {
                stash_loaded(z)?;
            }
            fs::write(&list, format!("{PLACEHOLDER}\r\n"))?;
        }
        "any" => {
            if current == "loaded" {
                stash_loaded(z)?;
            }
            fs::write(&list, "")?;
        }
        "loaded" => {
            let backup = backup_file(z);
            if !backup.exists() {
                anyhow::bail!("NO_IPSET_BACKUP");
            }
            if list.exists() {
                fs::remove_file(&list)?;
            }
            fs::rename(&backup, &list)?;
        }
        _ => anyhow::bail!("invalid ipset mode"),
    }
    Ok(status(z))
}

/// Downloads the fresh list (service.bat menu item "Update IPSet List").
pub async fn update_list(z: &Path) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("Fork-Zapret")
        .timeout(std::time::Duration::from_secs(20))
        .build()?;
    let body = client.get(IPSET_URL).send().await?.error_for_status()?.text().await?;
    if body.trim().is_empty() {
        anyhow::bail!("empty ipset list received");
    }
    fs::write(list_file(z), body)?;
    Ok(status(z))
}
