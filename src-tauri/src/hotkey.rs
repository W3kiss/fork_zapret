//! Global hotkey (default Ctrl+Shift+Z) toggling zapret even when minimised to tray.

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

/// Re-registers the toggle hotkey. An empty string disables it.
pub fn apply(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    let hk = hotkey.trim();
    if hk.is_empty() {
        return Ok(());
    }
    let shortcut: Shortcut = hk.parse().map_err(|e| format!("invalid hotkey: {e}"))?;
    gs.register(shortcut).map_err(|e| format!("hotkey: {e}"))
}
