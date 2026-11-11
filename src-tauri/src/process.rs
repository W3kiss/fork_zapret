//! winws.exe lifecycle: spawn directly (no cmd.exe), hidden, with log capture.

use crate::util::{hidden_cmd, run_hidden};
use serde::Serialize;
use std::fs::File;
use std::path::Path;
use std::process::{Child, Stdio};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ConnState {
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusPayload {
    pub state: ConnState,
    pub strategy: Option<String>,
    /// Unix ms when the connection was established.
    pub since: Option<i64>,
    pub error: Option<String>,
    /// "manual" | "smart" | "autopick" | "hotkey" | "tray" | "autostart"
    pub origin: Option<String>,
}

impl Default for StatusPayload {
    fn default() -> Self {
        Self { state: ConnState::Disconnected, strategy: None, since: None, error: None, origin: None }
    }
}

#[derive(Default)]
pub struct Engine {
    child: Mutex<Option<Child>>,
    pub status: Mutex<StatusPayload>,
}

impl Engine {
    pub fn snapshot(&self) -> StatusPayload {
        self.status.lock().unwrap().clone()
    }

    pub fn set_status(&self, s: StatusPayload) {
        *self.status.lock().unwrap() = s;
    }

    pub fn is_running(&self) -> bool {
        let mut guard = self.child.lock().unwrap();
        match guard.as_mut() {
            Some(c) => matches!(c.try_wait(), Ok(None)),
            None => false,
        }
    }

    /// Returns `Some(exit_code)` if our child exited on its own since the last check.
    pub fn poll_exit(&self) -> Option<i32> {
        let mut guard = self.child.lock().unwrap();
        let exited = match guard.as_mut() {
            Some(c) => match c.try_wait() {
                Ok(Some(st)) => Some(st.code().unwrap_or(-1)),
                Ok(None) => None,
                Err(_) => Some(-1),
            },
            None => None,
        };
        if exited.is_some() {
            *guard = None;
        }
        exited
    }

    pub fn spawn(&self, zapret: &Path, args: &[String], log: &Path) -> anyhow::Result<()> {
        self.kill();
        kill_all_winws();
        let bin = zapret.join("bin");
        let exe = bin.join("winws.exe");
        if !exe.exists() {
            anyhow::bail!("winws.exe not found: {}", exe.display());
        }
        let out = File::create(log)?;
        let err = out.try_clone()?;
        let child = hidden_cmd(&exe)
            .args(args)
            .current_dir(&bin)
            .stdout(Stdio::from(out))
            .stderr(Stdio::from(err))
            .spawn()?;
        *self.child.lock().unwrap() = Some(child);
        Ok(())
    }

    pub fn kill(&self) {
        if let Some(mut c) = self.child.lock().unwrap().take() {
            let _ = c.kill();
            let _ = c.wait();
        }
    }

    pub fn stop(&self) {
        self.kill();
        kill_all_winws();
    }
}

/// Kills any winws.exe (including ones started by the original .bat files).
pub fn kill_all_winws() {
    let _ = run_hidden("taskkill", ["/IM", "winws.exe", "/F"]);
}

#[allow(dead_code)]
pub fn winws_running_anywhere() -> bool {
    run_hidden("tasklist", ["/FI", "IMAGENAME eq winws.exe", "/NH"])
        .map(|o| String::from_utf8_lossy(&o.stdout).to_ascii_lowercase().contains("winws.exe"))
        .unwrap_or(false)
}

/// Mirrors `service.bat :tcp_enable` — required by `--dpi-desync-fooling=ts`.
pub fn enable_tcp_timestamps() {
    let _ = run_hidden("netsh", ["interface", "tcp", "set", "global", "timestamps=enabled"]);
}

/// Stops and removes the WinDivert driver service so binaries can be replaced
/// (mirrors `service.bat :service_remove`).
pub fn release_driver() {
    for svc in ["WinDivert", "WinDivert14"] {
        let _ = run_hidden("sc", ["stop", svc]);
        let _ = run_hidden("sc", ["delete", svc]);
    }
}

/// True if the Flowseal `zapret` Windows service (installed via service.bat) is running.
pub fn zapret_service_running() -> bool {
    run_hidden("sc", ["query", "zapret"])
        .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).contains("RUNNING"))
        .unwrap_or(false)
}

/// Stops (but does not delete) the service.bat-installed service so it does not
/// fight with our winws.exe over WinDivert.
pub fn stop_zapret_service() {
    let _ = run_hidden("sc", ["stop", "zapret"]);
}

pub fn is_elevated() -> bool {
    run_hidden("net", ["session"]).map(|o| o.status.success()).unwrap_or(false)
}

pub fn read_log_tail(log: &Path, max: usize) -> String {
    let text = crate::util::read_text(log).unwrap_or_default();
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    let start = lines.len().saturating_sub(max);
    lines[start..].join("\n")
}
