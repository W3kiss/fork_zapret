//! Fork Zapret — desktop launcher for zapret (Flowseal).
//! Author: W3kiss — https://github.com/W3kiss

mod appupdate;
mod autopick;
mod autostart;
mod commands;
mod config;
mod controller;
mod diagnostics;
mod game_filter;
mod hotkey;
mod ipset;
mod lists;
mod process;
mod schedule;
mod smart_mode;
mod state;
mod stats;
mod strategy;
mod tray;
mod updater;
mod util;

use state::{AppPaths, AppState};
use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_global_shortcut::ShortcutState;

/// Debug builds use an `asInvoker` manifest so `tauri dev` can start them from a
/// normal terminal. Without admin rights we re-launch ourselves elevated (UAC prompt)
/// and keep this process alive until the elevated copy exits, so the dev server stays up.
#[cfg(debug_assertions)]
fn ensure_elevated_dev() {
    if process::is_elevated() || std::env::args().any(|a| a == "--elevated") {
        return;
    }
    let Ok(exe) = std::env::current_exe() else { return };
    let exe = exe.to_string_lossy().replace('\'', "''");
    let script = format!(
        "$p = Start-Process -FilePath '{exe}' -ArgumentList '--elevated' -Verb RunAs -PassThru -Wait; exit $p.ExitCode"
    );
    let status = util::hidden_cmd("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
        .status();
    match status {
        // Elevated copy ran (or the user declined UAC) — this unelevated process is done.
        Ok(s) => std::process::exit(s.code().unwrap_or(0)),
        Err(e) => eprintln!("elevation failed: {e}; continuing without admin rights"),
    }
}

pub fn run() {
    #[cfg(debug_assertions)]
    ensure_elevated_dev();

    let launched_by_autostart = std::env::args().any(|a| a == "--autostart");

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| tray::show_main(app)))
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if matches!(event.state(), ShortcutState::Pressed) {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = controller::toggle(&app, "hotkey").await;
                        });
                    }
                })
                .build(),
        )
        .setup(move |app| {
            let handle = app.handle().clone();
            let paths = AppPaths::init(&handle)?;
            let cfg = config::load(&paths.config_file);
            if let Err(e) = state::seed_zapret(&handle, &paths.default_zapret) {
                eprintln!("zapret seed: {e}");
            }
            let hotkey_str = cfg.hotkey.clone();
            let auto_connect = cfg.auto_connect;
            let start_hidden = launched_by_autostart && cfg.minimize_to_tray;
            app.manage(AppState::new(paths, cfg));

            tray::create(&handle)?;
            if let Err(e) = hotkey::apply(&handle, &hotkey_str) {
                eprintln!("{e}");
            }
            controller::spawn_watchdog(handle.clone());
            smart_mode::spawn(handle.clone());
            schedule::spawn(handle.clone());

            if let Some(w) = app.get_webview_window("main") {
                if !start_hidden {
                    let _ = w.show();
                }
            }

            if auto_connect {
                let h = handle.clone();
                tauri::async_runtime::spawn(async move {
                    let origin = if launched_by_autostart { "autostart" } else { "manual" };
                    let _ = controller::connect(&h, None, origin).await;
                });
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let st = window.state::<AppState>();
                if st.cfg().minimize_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                } else {
                    st.engine.stop();
                    window.app_handle().exit(0);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::save_config,
            commands::reset_settings,
            commands::list_strategies,
            commands::preview_args,
            commands::connect,
            commands::disconnect,
            commands::toggle,
            commands::reconnect_if_running,
            commands::zapret_full_stop,
            commands::set_game_filter,
            commands::set_ipset_mode,
            commands::update_ipset_list,
            commands::list_read,
            commands::list_add,
            commands::list_remove,
            commands::autopick_start,
            commands::autopick_cancel,
            commands::check_update,
            commands::install_update,
            commands::list_backups,
            commands::create_backup,
            commands::delete_backup,
            commands::restore_backup,
            commands::get_avatar,
            commands::open_url,
            commands::open_zapret_folder,
            commands::set_tray_labels,
            commands::quit_app,
            commands::winws_log,
            commands::apply_profile,
            commands::run_diagnostics,
            commands::get_stats,
            commands::reset_stats,
            commands::check_app_update,
            commands::install_app_update,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Fork Zapret");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            if let Some(st) = handle.try_state::<AppState>() {
                st.engine.stop();
            }
        }
    });
}
