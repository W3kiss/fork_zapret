//! System diagnostics, mirroring the checks in Flowseal `service.bat`.

use crate::process;
use crate::util::run_hidden;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub name: String,
    /// ok | warn | fail
    pub status: String,
    pub detail: String,
}

fn check(name: &str, status: &str, detail: &str) -> Check {
    Check { name: name.into(), status: status.into(), detail: detail.into() }
}

fn sc_contains(service: &str, needle: &str) -> bool {
    run_hidden("sc", ["query", service])
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(needle))
        .unwrap_or(false)
}

pub fn run(zapret: &Path) -> Vec<Check> {
    let mut out = Vec::new();

    // Administrator rights (WinDivert needs them).
    out.push(if process::is_elevated() {
        check("admin", "ok", "Запущено с правами администратора")
    } else {
        check("admin", "fail", "Нет прав администратора — WinDivert не запустится")
    });

    // winws.exe present.
    out.push(if zapret.join("bin").join("winws.exe").exists() {
        check("winws", "ok", "winws.exe на месте")
    } else {
        check("winws", "fail", "winws.exe не найден в bin")
    });

    // WinDivert driver file.
    out.push(if zapret.join("bin").join("WinDivert64.sys").exists() {
        check("windivert", "ok", "Драйвер WinDivert64.sys на месте")
    } else {
        check("windivert", "fail", "WinDivert64.sys не найден в bin")
    });

    // Base Filtering Engine must be running.
    out.push(if sc_contains("BFE", "RUNNING") {
        check("bfe", "ok", "Служба BFE работает")
    } else {
        check("bfe", "fail", "Служба BFE остановлена — zapret не сможет работать")
    });

    // TCP timestamps (needed by --dpi-desync-fooling=ts).
    let ts = run_hidden("netsh", ["interface", "tcp", "show", "global"])
        .map(|o| String::from_utf8_lossy(&o.stdout).to_lowercase())
        .unwrap_or_default();
    out.push(if ts.contains("timestamps") && ts.contains("enabled") {
        check("timestamps", "ok", "TCP timestamps включены")
    } else {
        check("timestamps", "warn", "TCP timestamps выключены — часть стратегий слабее")
    });

    // System proxy — a running proxy can interfere.
    let proxy = run_hidden(
        "reg",
        ["query", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings", "/v", "ProxyEnable"],
    )
    .map(|o| String::from_utf8_lossy(&o.stdout).to_lowercase())
    .unwrap_or_default();
    out.push(if proxy.contains("0x1") {
        check("proxy", "warn", "Включён системный прокси — может мешать")
    } else {
        check("proxy", "ok", "Системный прокси выключен")
    });

    // Conflicting Flowseal service installed via service.bat.
    out.push(if process::zapret_service_running() {
        check("service", "warn", "Служба zapret (service.bat) запущена — возможен конфликт за WinDivert")
    } else {
        check("service", "ok", "Служба zapret не мешает")
    });

    // Another process holding WinDivert that is not ours.
    let winws_running = process::winws_running_anywhere();
    let windivert_up = sc_contains("WinDivert", "RUNNING") || sc_contains("WinDivert14", "RUNNING");
    out.push(if windivert_up && !winws_running {
        check("driver", "warn", "WinDivert загружен, но winws не запущен — используйте «Полностью выключить zapret»")
    } else {
        check("driver", "ok", "Конфликтов драйвера не видно")
    });

    out
}
