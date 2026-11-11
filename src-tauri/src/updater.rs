//! zapret updates from GitHub Releases, backups and rollback.

use crate::controller;
use crate::state::AppState;
use crate::util::{copy_dir_all, copy_dir_filtered, dir_size, read_text, version_newer};
use crate::{ipset, lists, process};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager};

pub const PROGRESS_EVENT: &str = "update://progress";
const RELEASES_API: &str = "https://api.github.com/repos/Flowseal/zapret-discord-youtube/releases/latest";
const MAX_BACKUPS: usize = 5;

/// User files that must survive an update.
const PRESERVE: &[&str] = &[
    "lists/list-general-user.txt",
    "lists/list-exclude-user.txt",
    "lists/ipset-exclude-user.txt",
    "utils/game_filter.enabled",
    "utils/check_updates.enabled",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub local: String,
    pub latest: String,
    pub has_update: bool,
    pub url: String,
    pub notes: String,
    pub asset: Option<String>,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    /// backup | download | extract | apply | done | error
    pub phase: String,
    pub progress: f64,
    pub downloaded: u64,
    pub total: u64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub id: String,
    pub version: String,
    pub created: i64,
    pub size: u64,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    body: Option<String>,
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

pub fn local_version(zapret: &Path) -> String {
    let text = read_text(&zapret.join("service.bat")).unwrap_or_default();
    for line in text.lines() {
        if let Some(i) = crate::util::find_ci(line, "LOCAL_VERSION=") {
            return line[i + "LOCAL_VERSION=".len()..].trim().trim_end_matches('"').trim().to_string();
        }
    }
    "unknown".into()
}

fn client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent("Fork-Zapret (https://github.com/W3kiss)")
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
}

async fn fetch_release() -> anyhow::Result<Release> {
    let r = client()?
        .get(RELEASES_API)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?
        .json::<Release>()
        .await?;
    Ok(r)
}

fn pick_asset(r: &Release) -> Option<Asset> {
    r.assets.iter().find(|a| a.name.to_ascii_lowercase().ends_with(".zip")).cloned()
}

pub async fn check(zapret: &Path) -> anyhow::Result<UpdateInfo> {
    let local = local_version(zapret);
    let r = fetch_release().await?;
    let latest = r.tag_name.trim_start_matches(['v', 'V']).to_string();
    let asset = pick_asset(&r);
    Ok(UpdateInfo {
        has_update: version_newer(&latest, &local),
        local,
        latest,
        url: r.html_url.clone(),
        notes: r.body.clone().unwrap_or_default(),
        size: asset.as_ref().map(|a| a.size).unwrap_or(0),
        asset: asset.map(|a| a.name),
    })
}

fn emit(app: &AppHandle, phase: &str, progress: f64, downloaded: u64, total: u64, message: Option<String>) {
    let _ = app.emit(
        PROGRESS_EVENT,
        UpdateProgress { phase: phase.into(), progress, downloaded, total, message },
    );
}

// ---------------------------------------------------------------- backups

pub fn create_backup(st: &AppState) -> anyhow::Result<BackupInfo> {
    let zapret = st.zapret_dir();
    let version = local_version(&zapret);
    let ts = chrono::Local::now();
    let id = format!("{}_v{}", ts.format("%Y%m%d-%H%M%S"), version.replace(|c: char| !c.is_ascii_alphanumeric() && c != '.', "_"));
    let dest = st.paths.backups_dir.join(&id);
    copy_dir_all(&zapret, &dest)?;
    prune_backups(&st.paths.backups_dir);
    Ok(BackupInfo { id, version, created: ts.timestamp_millis(), size: dir_size(&dest) })
}

pub fn list_backups(dir: &Path) -> Vec<BackupInfo> {
    let mut out: Vec<BackupInfo> = fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| {
                    let id = e.file_name().to_string_lossy().to_string();
                    let created = e
                        .metadata()
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_millis() as i64)
                        .unwrap_or(0);
                    BackupInfo { version: local_version(&e.path()), size: dir_size(&e.path()), created, id }
                })
                .collect()
        })
        .unwrap_or_default();
    out.sort_by(|a, b| b.id.cmp(&a.id));
    out
}

fn prune_backups(dir: &Path) {
    for old in list_backups(dir).into_iter().skip(MAX_BACKUPS) {
        let _ = fs::remove_dir_all(dir.join(old.id));
    }
}

pub fn delete_backup(st: &AppState, id: &str) -> anyhow::Result<()> {
    if id.contains(['/', '\\']) || id.contains("..") {
        anyhow::bail!("invalid backup id");
    }
    fs::remove_dir_all(st.paths.backups_dir.join(id))?;
    Ok(())
}

/// Replaces the zapret directory with a backup (rollback).
pub async fn restore_backup(app: &AppHandle, id: &str) -> anyhow::Result<String> {
    if id.contains(['/', '\\']) || id.contains("..") {
        anyhow::bail!("invalid backup id");
    }
    let st = app.state::<AppState>();
    let src = st.paths.backups_dir.join(id);
    if !src.join("bin").join("winws.exe").exists() {
        anyhow::bail!("backup is damaged");
    }
    let _g = st.op_lock.lock().await;
    st.updating.store(true, Ordering::SeqCst);
    controller::stop_locked(app, "update");
    process::release_driver();
    let zapret = st.zapret_dir();
    let res = (|| -> anyhow::Result<()> {
        clear_dir(&zapret)?;
        copy_dir_all(&src, &zapret)?;
        Ok(())
    })();
    st.updating.store(false, Ordering::SeqCst);
    res?;
    Ok(local_version(&zapret))
}

fn clear_dir(dir: &Path) -> std::io::Result<()> {
    if !dir.exists() {
        return fs::create_dir_all(dir);
    }
    for e in fs::read_dir(dir)?.flatten() {
        let p = e.path();
        if p.is_dir() {
            fs::remove_dir_all(&p)?;
        } else {
            fs::remove_file(&p)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- install

fn find_root(dir: &Path, depth: u8) -> Option<PathBuf> {
    if dir.join("service.bat").exists() && dir.join("bin").exists() {
        return Some(dir.to_path_buf());
    }
    if depth == 0 {
        return None;
    }
    fs::read_dir(dir).ok()?.flatten().filter(|e| e.path().is_dir()).find_map(|e| find_root(&e.path(), depth - 1))
}

fn extract_zip(archive: &Path, dest: &Path) -> anyhow::Result<()> {
    let file = fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(rel) = entry.enclosed_name() else { continue };
        let out = dest.join(rel);
        if entry.is_dir() {
            fs::create_dir_all(&out)?;
        } else {
            if let Some(p) = out.parent() {
                fs::create_dir_all(p)?;
            }
            let mut f = fs::File::create(&out)?;
            std::io::copy(&mut entry, &mut f)?;
        }
    }
    Ok(())
}

fn entries_of(path: &Path) -> Vec<String> {
    read_text(path)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

pub async fn install(app: &AppHandle) -> anyhow::Result<String> {
    let st = app.state::<AppState>();
    let zapret = st.zapret_dir();
    let release = fetch_release().await?;
    let asset = pick_asset(&release).ok_or_else(|| anyhow::anyhow!("release has no .zip asset"))?;

    let _g = st.op_lock.lock().await;
    st.updating.store(true, Ordering::SeqCst);
    let result = install_inner(app, &st, &zapret, &asset).await;
    st.updating.store(false, Ordering::SeqCst);
    match &result {
        Ok(v) => emit(app, "done", 1.0, 0, 0, Some(v.clone())),
        Err(e) => emit(app, "error", 0.0, 0, 0, Some(e.to_string())),
    }
    result
}

async fn install_inner(app: &AppHandle, st: &AppState, zapret: &Path, asset: &Asset) -> anyhow::Result<String> {
    // 1. stop & backup
    emit(app, "backup", 0.0, 0, 0, None);
    controller::stop_locked(app, "update");
    process::release_driver();
    create_backup(st)?;

    // 2. download with progress
    let tmp = st.paths.data_dir.join("update-tmp");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp)?;
    let archive = tmp.join("release.zip");
    let resp = client()?.get(&asset.browser_download_url).send().await?.error_for_status()?;
    let total = resp.content_length().unwrap_or(asset.size);
    let mut file = fs::File::create(&archive)?;
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_emit = 0u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        if downloaded - last_emit > 64 * 1024 || downloaded == total {
            last_emit = downloaded;
            let p = if total > 0 { downloaded as f64 / total as f64 } else { 0.0 };
            emit(app, "download", p, downloaded, total, None);
        }
    }
    drop(file);

    // 3. extract
    emit(app, "extract", 0.0, 0, 0, None);
    let extract_dir = tmp.join("x");
    {
        let (a, d) = (archive.clone(), extract_dir.clone());
        tokio::task::spawn_blocking(move || extract_zip(&a, &d)).await??;
    }
    let new_root = find_root(&extract_dir, 3).ok_or_else(|| anyhow::anyhow!("archive does not look like a zapret build"))?;

    // 4. apply, preserving user files
    emit(app, "apply", 0.5, 0, 0, None);
    let prev_ipset = ipset::status(zapret);
    let old_general = entries_of(&zapret.join("lists").join("list-general.txt"));
    let new_general = entries_of(&new_root.join("lists").join("list-general.txt"));

    // Remove strategy .bat files that no longer exist in the new release.
    if let Ok(rd) = fs::read_dir(zapret) {
        for e in rd.flatten() {
            let name = e.file_name();
            let lower = name.to_string_lossy().to_ascii_lowercase();
            if lower.ends_with(".bat") && !new_root.join(&name).exists() {
                let _ = fs::remove_file(e.path());
            }
        }
    }

    let z = zapret.to_path_buf();
    copy_dir_filtered(&new_root, zapret, &move |rel: &Path| {
        let r = rel.to_string_lossy().replace('\\', "/");
        PRESERVE.iter().any(|p| *p == r) && z.join(rel).exists()
    })?;

    // Migrate custom domains the user added to list-general.txt into the -user list.
    let custom: Vec<String> = old_general
        .into_iter()
        .filter(|d| !new_general.iter().any(|n| n.eq_ignore_ascii_case(d)))
        .collect();
    lists::ensure_user_lists(zapret);
    if !custom.is_empty() {
        let _ = lists::add(zapret, "list-general-user.txt", &custom);
    }

    // Restore the previous IPSet mode.
    let _ = ipset::set_mode(zapret, &prev_ipset);

    let _ = fs::remove_dir_all(&tmp);
    Ok(local_version(zapret))
}
