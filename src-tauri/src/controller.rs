//! High-level connect / disconnect used by commands, tray, hotkey, smart mode and auto-pick.

use crate::process::{self, ConnState, StatusPayload};
use crate::state::AppState;
use crate::{game_filter, lists, strategy, tray};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub const STATUS_EVENT: &str = "zapret://status";

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub fn publish(app: &AppHandle, status: StatusPayload) {
    let st = app.state::<AppState>();
    st.engine.set_status(status.clone());
    tray::set_connected(app, status.state == ConnState::Connected);
    let _ = app.emit(STATUS_EVENT, status);
}

/// Starts winws.exe for `file` without taking the op lock (caller holds it).
pub async fn start_locked(app: &AppHandle, file: &str, origin: &str) -> Result<StatusPayload, String> {
    let st = app.state::<AppState>();
    if st.updating.load(Ordering::SeqCst) {
        return Err("UPDATING".into());
    }
    let zapret = st.zapret_dir();
    publish(
        app,
        StatusPayload {
            state: ConnState::Connecting,
            strategy: Some(file.to_string()),
            origin: Some(origin.into()),
            ..Default::default()
        },
    );

    let gf = game_filter::load(&zapret);
    let result: Result<(), String> = (|| {
        lists::ensure_user_lists(&zapret);
        process::enable_tcp_timestamps();
        if process::zapret_service_running() {
            // The service.bat-installed service would fight over WinDivert.
            process::stop_zapret_service();
        }
        let args = strategy::parse_args(&zapret, file, &gf).map_err(|e| e.to_string())?;
        st.engine.spawn(&zapret, &args, &st.paths.winws_log).map_err(|e| e.to_string())
    })();

    if let Err(e) = result {
        let s = StatusPayload { error: Some(e.clone()), origin: Some(origin.into()), ..Default::default() };
        publish(app, s);
        return Err(e);
    }

    // Give WinDivert time to load; detect immediate failures (bad args, driver error).
    tokio::time::sleep(Duration::from_millis(900)).await;
    if let Some(code) = st.engine.poll_exit() {
        let log = process::read_log_tail(&st.paths.winws_log, 6);
        let msg = if log.is_empty() { format!("winws.exe exited with code {code}") } else { log };
        publish(app, StatusPayload { error: Some(msg.clone()), origin: Some(origin.into()), ..Default::default() });
        return Err(msg);
    }

    let status = StatusPayload {
        state: ConnState::Connected,
        strategy: Some(file.to_string()),
        since: Some(now_ms()),
        error: None,
        origin: Some(origin.into()),
    };
    publish(app, status.clone());
    Ok(status)
}

pub fn stop_locked(app: &AppHandle, origin: &str) -> StatusPayload {
    let st = app.state::<AppState>();
    st.engine.stop();
    let s = StatusPayload { origin: Some(origin.into()), ..Default::default() };
    publish(app, s.clone());
    s
}

pub async fn connect(app: &AppHandle, strategy: Option<String>, origin: &str) -> Result<StatusPayload, String> {
    let st = app.state::<AppState>();
    if st.autopick_running.load(Ordering::SeqCst) {
        return Err("AUTOPICK_RUNNING".into());
    }
    let _g = st.op_lock.lock().await;
    let file = match strategy {
        Some(f) => {
            st.update_cfg(|c| c.strategy = f.clone());
            f
        }
        None => st.cfg().strategy,
    };
    if origin != "smart" {
        st.smart_suppressed.store(false, Ordering::SeqCst);
    }
    start_locked(app, &file, origin).await
}

pub async fn disconnect(app: &AppHandle, origin: &str) -> Result<StatusPayload, String> {
    let st = app.state::<AppState>();
    if st.autopick_running.load(Ordering::SeqCst) {
        st.autopick_cancel.store(true, Ordering::SeqCst);
    }
    let _g = st.op_lock.lock().await;
    if origin != "smart" && st.cfg().smart_mode {
        st.smart_suppressed.store(true, Ordering::SeqCst);
    }
    Ok(stop_locked(app, origin))
}

/// Hard stop: kills winws.exe AND unloads the WinDivert driver, leaving the app open.
/// Used to fully release WinDivert (e.g. so it stops fighting a VPN's TUN adapter).
pub async fn full_stop(app: &AppHandle, origin: &str) -> StatusPayload {
    let st = app.state::<AppState>();
    if st.autopick_running.load(Ordering::SeqCst) {
        st.autopick_cancel.store(true, Ordering::SeqCst);
    }
    let _g = st.op_lock.lock().await;
    // Keep Smart Mode from immediately restarting winws after a deliberate hard stop.
    if st.cfg().smart_mode {
        st.smart_suppressed.store(true, Ordering::SeqCst);
    }
    st.engine.stop();
    process::release_driver();
    let s = StatusPayload { origin: Some(origin.into()), ..Default::default() };
    publish(app, s.clone());
    s
}

pub async fn toggle(app: &AppHandle, origin: &str) -> Result<StatusPayload, String> {
    let st = app.state::<AppState>();
    let running = st.engine.is_running();
    if running {
        disconnect(app, origin).await
    } else {
        connect(app, None, origin).await
    }
}

/// Background watcher: detects winws.exe crashing or being killed externally.
pub fn spawn_watchdog(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(1500)).await;
            let st = app.state::<AppState>();
            if st.autopick_running.load(Ordering::SeqCst) || st.updating.load(Ordering::SeqCst) {
                continue;
            }
            let was_connected = st.engine.snapshot().state == ConnState::Connected;
            if let Some(code) = st.engine.poll_exit() {
                if was_connected {
                    let log = process::read_log_tail(&st.paths.winws_log, 4);
                    let msg = if log.is_empty() { format!("winws.exe exited ({code})") } else { log };
                    publish(&app, StatusPayload { error: Some(msg), ..Default::default() });
                }
            }
        }
    });
}
