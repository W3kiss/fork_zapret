//! Tauri commands exposed to the Svelte frontend.

use crate::config::{self, AppConfig};
use crate::game_filter::{self, GameFilter};
use crate::process::{self, StatusPayload};
use crate::state::AppState;
use crate::strategy::{self, StrategyInfo};
use crate::updater::{self, BackupInfo, UpdateInfo};
use crate::{autopick, autostart, controller, hotkey, ipset, lists, tray};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

type R<T> = Result<T, String>;

fn e<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub config: AppConfig,
    pub status: StatusPayload,
    pub strategies: Vec<StrategyInfo>,
    pub zapret_version: String,
    pub zapret_dir: String,
    pub game_filter: GameFilter,
    pub ipset_mode: String,
    pub ipset_has_backup: bool,
    pub is_admin: bool,
    pub app_version: String,
    pub autopick_running: bool,
}

#[tauri::command]
pub async fn get_snapshot(app: AppHandle) -> R<Snapshot> {
    let st = app.state::<AppState>();
    let z = st.zapret_dir();
    let mut cfg = st.cfg();
    cfg.autostart = autostart::is_enabled();
    Ok(Snapshot {
        config: cfg,
        status: st.engine.snapshot(),
        strategies: strategy::list(&z),
        zapret_version: updater::local_version(&z),
        zapret_dir: z.to_string_lossy().to_string(),
        game_filter: game_filter::load(&z),
        ipset_mode: ipset::status(&z),
        ipset_has_backup: ipset::has_backup(&z),
        is_admin: process::is_elevated(),
        app_version: app.package_info().version.to_string(),
        autopick_running: st.autopick_running.load(std::sync::atomic::Ordering::SeqCst),
    })
}

/// Saves the full config and applies side effects (hotkey, autostart).
#[tauri::command]
pub async fn save_config(app: AppHandle, config: AppConfig) -> R<AppConfig> {
    let st = app.state::<AppState>();
    let prev = st.cfg();
    if prev.hotkey != config.hotkey {
        hotkey::apply(&app, &config.hotkey)?;
    }
    if prev.autostart != config.autostart || autostart::is_enabled() != config.autostart {
        autostart::set(config.autostart).map_err(e)?;
    }
    Ok(st.update_cfg(|c| *c = config))
}

#[tauri::command]
pub fn reset_settings(app: AppHandle, st: State<'_, AppState>) -> R<AppConfig> {
    let _ = autostart::disable();
    let mut def = AppConfig::default();
    def.onboarded = false;
    let _ = hotkey::apply(&app, &def.hotkey);
    let _ = std::fs::remove_file(&st.paths.avatar_file);
    config::save(&st.paths.config_file, &def).map_err(e)?;
    Ok(st.update_cfg(|c| *c = def))
}

#[tauri::command]
pub fn list_strategies(st: State<'_, AppState>) -> Vec<StrategyInfo> {
    strategy::list(&st.zapret_dir())
}

/// Returns the parsed winws.exe argument list for a strategy (shown in the UI).
#[tauri::command]
pub fn preview_args(st: State<'_, AppState>, file: String) -> R<Vec<String>> {
    let z = st.zapret_dir();
    strategy::parse_args(&z, &file, &game_filter::load(&z)).map_err(e)
}

#[tauri::command]
pub async fn connect(app: AppHandle, strategy: Option<String>) -> R<StatusPayload> {
    controller::connect(&app, strategy, "manual").await
}

#[tauri::command]
pub async fn disconnect(app: AppHandle) -> R<StatusPayload> {
    controller::disconnect(&app, "manual").await
}

#[tauri::command]
pub async fn toggle(app: AppHandle) -> R<StatusPayload> {
    controller::toggle(&app, "manual").await
}

/// Restarts the current connection (used after changing Game Filter / IPSet).
#[tauri::command]
pub async fn reconnect_if_running(app: AppHandle) -> R<StatusPayload> {
    let running = app.state::<AppState>().engine.is_running();
    if running {
        controller::connect(&app, None, "manual").await
    } else {
        Ok(app.state::<AppState>().engine.snapshot())
    }
}

/// Fully stops zapret: kills winws and unloads the WinDivert driver (app stays open).
#[tauri::command]
pub async fn zapret_full_stop(app: AppHandle) -> R<StatusPayload> {
    Ok(controller::full_stop(&app, "manual").await)
}

// ---------------------------------------------------------------- filters

#[tauri::command]
pub fn set_game_filter(st: State<'_, AppState>, filter: GameFilter) -> R<GameFilter> {
    game_filter::save(&st.zapret_dir(), &filter).map_err(e)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IpsetState {
    pub mode: String,
    pub has_backup: bool,
}

#[tauri::command]
pub fn set_ipset_mode(st: State<'_, AppState>, mode: String) -> R<IpsetState> {
    let z = st.zapret_dir();
    let mode = ipset::set_mode(&z, &mode).map_err(e)?;
    Ok(IpsetState { mode, has_backup: ipset::has_backup(&z) })
}

#[tauri::command]
pub async fn update_ipset_list(app: AppHandle) -> R<IpsetState> {
    let z = app.state::<AppState>().zapret_dir();
    let mode = ipset::update_list(&z).await.map_err(e)?;
    Ok(IpsetState { mode, has_backup: ipset::has_backup(&z) })
}

// ---------------------------------------------------------------- lists

#[tauri::command]
pub fn list_read(st: State<'_, AppState>, name: String) -> R<Vec<String>> {
    lists::read_entries(&st.zapret_dir(), &name).map_err(e)
}

#[tauri::command]
pub fn list_add(st: State<'_, AppState>, name: String, entries: Vec<String>) -> R<Vec<String>> {
    lists::add(&st.zapret_dir(), &name, &entries).map_err(e)
}

#[tauri::command]
pub fn list_remove(st: State<'_, AppState>, name: String, entry: String) -> R<Vec<String>> {
    lists::remove(&st.zapret_dir(), &name, &entry).map_err(e)
}

// ---------------------------------------------------------------- auto-pick

#[tauri::command]
pub async fn autopick_start(app: AppHandle) -> R<Option<String>> {
    autopick::run(app).await
}

#[tauri::command]
pub fn autopick_cancel(st: State<'_, AppState>) {
    st.autopick_cancel.store(true, std::sync::atomic::Ordering::SeqCst);
}

// ---------------------------------------------------------------- updates

#[tauri::command]
pub async fn check_update(app: AppHandle) -> R<UpdateInfo> {
    let z = app.state::<AppState>().zapret_dir();
    updater::check(&z).await.map_err(e)
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> R<String> {
    updater::install(&app).await.map_err(e)
}

#[tauri::command]
pub fn list_backups(st: State<'_, AppState>) -> Vec<BackupInfo> {
    updater::list_backups(&st.paths.backups_dir)
}

#[tauri::command]
pub fn create_backup(st: State<'_, AppState>) -> R<BackupInfo> {
    updater::create_backup(&st).map_err(e)
}

#[tauri::command]
pub fn delete_backup(st: State<'_, AppState>, id: String) -> R<()> {
    updater::delete_backup(&st, &id).map_err(e)
}

#[tauri::command]
pub async fn restore_backup(app: AppHandle, id: String) -> R<String> {
    updater::restore_backup(&app, &id).await.map_err(e)
}

// ---------------------------------------------------------------- misc

const AVATAR_URL: &str = "https://github.com/W3kiss.png?size=256";

/// Returns the developer avatar as a data URL. `refresh=false` returns the cached
/// copy instantly; `refresh=true` re-downloads it (cache-busted) and updates the cache.
#[tauri::command]
pub async fn get_avatar(app: AppHandle, refresh: bool) -> R<Option<String>> {
    let path = app.state::<AppState>().paths.avatar_file.clone();
    let to_url = |b: &[u8]| format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(b));
    if !refresh {
        return Ok(std::fs::read(&path).ok().map(|b| to_url(&b)));
    }
    let url = format!("{AVATAR_URL}&t={}", chrono::Utc::now().timestamp());
    let client = reqwest::Client::builder()
        .user_agent("Fork-Zapret")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(e)?;
    match client.get(url).header("Cache-Control", "no-cache").send().await.and_then(|r| r.error_for_status()) {
        Ok(resp) => {
            let bytes = resp.bytes().await.map_err(e)?;
            if bytes.len() > 100 {
                let _ = std::fs::write(&path, &bytes);
                return Ok(Some(to_url(&bytes)));
            }
            Ok(std::fs::read(&path).ok().map(|b| to_url(&b)))
        }
        Err(_) => Ok(std::fs::read(&path).ok().map(|b| to_url(&b))),
    }
}

#[tauri::command]
pub fn open_url(app: AppHandle, url: String) -> R<()> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("only http(s) links are allowed".into());
    }
    app.opener().open_url(url, None::<&str>).map_err(e)
}

#[tauri::command]
pub fn open_zapret_folder(app: AppHandle, st: State<'_, AppState>) -> R<()> {
    app.opener().open_path(st.zapret_dir().to_string_lossy().to_string(), None::<&str>).map_err(e)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayLabels {
    pub open: String,
    pub connect: String,
    pub disconnect: String,
    pub quit: String,
    pub tip_on: String,
    pub tip_off: String,
}

#[tauri::command]
pub fn set_tray_labels(app: AppHandle, labels: TrayLabels) {
    tray::set_labels(
        &app,
        tray::Labels {
            open: labels.open,
            connect: labels.connect,
            disconnect: labels.disconnect,
            quit: labels.quit,
            tip_on: labels.tip_on,
            tip_off: labels.tip_off,
        },
    );
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    tray::quit(&app);
}

#[tauri::command]
pub fn winws_log(st: State<'_, AppState>) -> String {
    process::read_log_tail(&st.paths.winws_log, 200)
}
