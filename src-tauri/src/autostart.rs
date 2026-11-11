//! Windows autostart via Task Scheduler with highest privileges (no UAC prompt on logon).

use crate::util::run_hidden;
use std::fs;

const TASK_NAME: &str = "Fork Zapret";

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn task_xml(exe: &str, user: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Author>W3kiss</Author>
    <Description>Fork Zapret autostart</Description>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
      <UserId>{user}</UserId>
      <Delay>PT5S</Delay>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>{user}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>true</AllowHardTerminate>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <IdleSettings>
      <StopOnIdleEnd>false</StopOnIdleEnd>
      <RestartOnIdle>false</RestartOnIdle>
    </IdleSettings>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <Hidden>false</Hidden>
    <RunOnlyIfIdle>false</RunOnlyIfIdle>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>7</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{exe}</Command>
      <Arguments>--autostart</Arguments>
    </Exec>
  </Actions>
</Task>
"#,
        exe = xml_escape(exe),
        user = xml_escape(user)
    )
}

pub fn enable() -> anyhow::Result<()> {
    let exe = std::env::current_exe()?.to_string_lossy().to_string();
    let user = match (std::env::var("USERDOMAIN"), std::env::var("USERNAME")) {
        (Ok(d), Ok(u)) => format!("{d}\\{u}"),
        (_, Ok(u)) => u,
        _ => anyhow::bail!("cannot resolve current user"),
    };
    // schtasks expects UTF-16 LE with BOM for XML definitions.
    let xml = task_xml(&exe, &user);
    let mut bytes: Vec<u8> = vec![0xFF, 0xFE];
    for u in xml.encode_utf16() {
        bytes.extend_from_slice(&u.to_le_bytes());
    }
    let path = std::env::temp_dir().join("fork-zapret-task.xml");
    fs::write(&path, bytes)?;
    let xml_path = path.to_string_lossy().to_string();
    let out = run_hidden("schtasks", ["/Create", "/TN", TASK_NAME, "/XML", xml_path.as_str(), "/F"])?;
    let _ = fs::remove_file(&path);
    if !out.status.success() {
        anyhow::bail!("schtasks: {}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(())
}

pub fn disable() -> anyhow::Result<()> {
    if is_enabled() {
        let out = run_hidden("schtasks", ["/Delete", "/TN", TASK_NAME, "/F"])?;
        if !out.status.success() {
            anyhow::bail!("schtasks: {}", String::from_utf8_lossy(&out.stderr).trim());
        }
    }
    Ok(())
}

pub fn is_enabled() -> bool {
    run_hidden("schtasks", ["/Query", "/TN", TASK_NAME]).map(|o| o.status.success()).unwrap_or(false)
}

pub fn set(enabled: bool) -> anyhow::Result<()> {
    if enabled {
        enable()
    } else {
        disable()
    }
}
