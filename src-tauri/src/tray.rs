//! System tray with dynamic connected / disconnected icon and context menu.

use crate::controller;
use std::sync::Mutex;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

const ICON_ON: &[u8] = include_bytes!("../icons/tray-on.png");
const ICON_OFF: &[u8] = include_bytes!("../icons/tray-off.png");

pub struct Labels {
    pub open: String,
    pub connect: String,
    pub disconnect: String,
    pub quit: String,
    pub tip_on: String,
    pub tip_off: String,
}

impl Default for Labels {
    fn default() -> Self {
        Self {
            open: "Открыть Fork Zapret".into(),
            connect: "Подключить".into(),
            disconnect: "Отключить".into(),
            quit: "Выход".into(),
            tip_on: "Fork Zapret — подключено".into(),
            tip_off: "Fork Zapret — отключено".into(),
        }
    }
}

pub struct TrayState {
    pub tray: TrayIcon<Wry>,
    pub open: MenuItem<Wry>,
    pub toggle: MenuItem<Wry>,
    pub quit: MenuItem<Wry>,
    pub labels: Mutex<Labels>,
    pub connected: Mutex<bool>,
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

pub fn quit(app: &AppHandle) {
    let st = app.state::<crate::state::AppState>();
    st.engine.stop();
    app.exit(0);
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let labels = Labels::default();
    let open = MenuItem::with_id(app, "open", &labels.open, true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, "toggle", &labels.connect, true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", &labels.quit, true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open, &toggle, &sep, &quit_i])?;

    let tray = TrayIconBuilder::with_id("main-tray")
        .icon(Image::from_bytes(ICON_OFF)?)
        .tooltip(&labels.tip_off)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_main(app),
            "toggle" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = controller::toggle(&app, "tray").await;
                });
            }
            "quit" => quit(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;

    app.manage(TrayState {
        tray,
        open,
        toggle,
        quit: quit_i,
        labels: Mutex::new(labels),
        connected: Mutex::new(false),
    });
    Ok(())
}

fn refresh(ts: &TrayState) {
    let connected = *ts.connected.lock().unwrap();
    let l = ts.labels.lock().unwrap();
    let _ = ts.open.set_text(&l.open);
    let _ = ts.quit.set_text(&l.quit);
    let _ = ts.toggle.set_text(if connected { &l.disconnect } else { &l.connect });
    let _ = ts.tray.set_tooltip(Some(if connected { &l.tip_on } else { &l.tip_off }));
    if let Ok(img) = Image::from_bytes(if connected { ICON_ON } else { ICON_OFF }) {
        let _ = ts.tray.set_icon(Some(img));
    }
}

pub fn set_connected(app: &AppHandle, connected: bool) {
    if let Some(ts) = app.try_state::<TrayState>() {
        *ts.connected.lock().unwrap() = connected;
        refresh(&ts);
    }
}

pub fn set_labels(app: &AppHandle, labels: Labels) {
    if let Some(ts) = app.try_state::<TrayState>() {
        *ts.labels.lock().unwrap() = labels;
        refresh(&ts);
    }
}
