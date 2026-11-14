//! Self-update: checks the launcher's own GitHub releases and installs a newer setup.exe.

use crate::util::version_newer;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

const RELEASES_API: &str = "https://api.github.com/repos/W3kiss/fork_zapret/releases/latest";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateInfo {
    pub current: String,
    pub latest: String,
    pub has_update: bool,
    pub url: String,
    pub setup: Option<String>,
    pub size: u64,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    assets: Vec<Asset>,
}

#[derive(Deserialize, Clone)]
struct Asset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

fn client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent("Fork-Zapret (https://github.com/W3kiss)")
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
}

async fn fetch() -> anyhow::Result<Release> {
    Ok(client()?
        .get(RELEASES_API)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?
        .json::<Release>()
        .await?)
}

fn setup_asset(r: &Release) -> Option<Asset> {
    r.assets
        .iter()
        .find(|a| {
            let n = a.name.to_ascii_lowercase();
            n.ends_with(".exe") && n.contains("setup")
        })
        .cloned()
}

pub async fn check(current: &str) -> anyhow::Result<AppUpdateInfo> {
    let r = fetch().await?;
    let latest = r.tag_name.trim_start_matches(['v', 'V']).to_string();
    let asset = setup_asset(&r);
    Ok(AppUpdateInfo {
        has_update: version_newer(&latest, current),
        current: current.to_string(),
        latest,
        url: r.html_url.clone(),
        size: asset.as_ref().map(|a| a.size).unwrap_or(0),
        setup: asset.map(|a| a.browser_download_url),
    })
}

/// Downloads the latest installer to a temp file and launches it, then the caller exits.
pub async fn download_installer(current: &str) -> anyhow::Result<std::path::PathBuf> {
    let r = fetch().await?;
    let latest = r.tag_name.trim_start_matches(['v', 'V']).to_string();
    if !version_newer(&latest, current) {
        anyhow::bail!("NO_APP_UPDATE");
    }
    let asset = setup_asset(&r).ok_or_else(|| anyhow::anyhow!("release has no setup.exe"))?;
    let bytes = client()?.get(&asset.browser_download_url).send().await?.error_for_status()?.bytes().await?;
    let path = std::env::temp_dir().join(&asset.name);
    let mut f = fs::File::create(&path)?;
    f.write_all(&bytes)?;
    Ok(path)
}
