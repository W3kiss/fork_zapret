//! Smart Mode: run winws.exe only while Discord / a browser is running.

use crate::controller;
use crate::process::ConnState;
use crate::state::AppState;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{ProcessesToUpdate, System};
use tauri::{AppHandle, Manager};

pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut sys = System::new();
        // Whether the current connection was started by Smart Mode.
        let started_by_smart = Arc::new(AtomicBool::new(false));
        loop {
            tokio::time::sleep(Duration::from_secs(3)).await;
            let st = app.state::<AppState>();
            let cfg = st.cfg();
            if !cfg.smart_mode {
                started_by_smart.store(false, Ordering::SeqCst);
                continue;
            }
            if st.autopick_running.load(Ordering::SeqCst) || st.updating.load(Ordering::SeqCst) {
                continue;
            }

            let targets: HashSet<String> = cfg.smart_processes.iter().map(|p| p.trim().to_ascii_lowercase()).collect();
            sys.refresh_processes(ProcessesToUpdate::All, true);
            let active = sys
                .processes()
                .values()
                .any(|p| targets.contains(&p.name().to_string_lossy().to_ascii_lowercase()));

            let status = st.engine.snapshot().state;
            let connected = status == ConnState::Connected && st.engine.is_running();

            if !active {
                // All target apps closed: clear manual suppression and stop what we started.
                st.smart_suppressed.store(false, Ordering::SeqCst);
                if connected && started_by_smart.load(Ordering::SeqCst) {
                    let _ = controller::disconnect(&app, "smart").await;
                    started_by_smart.store(false, Ordering::SeqCst);
                }
                continue;
            }

            if !connected
                && status != ConnState::Connecting
                && !st.smart_suppressed.load(Ordering::SeqCst)
                && controller::connect(&app, None, "smart").await.is_ok()
            {
                started_by_smart.store(true, Ordering::SeqCst);
            }
        }
    });
}
