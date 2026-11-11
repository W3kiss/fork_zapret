//! Strategy discovery and `.bat` → `winws.exe` argument parsing.
//!
//! Flowseal strategies look like:
//! ```bat
//! start "zapret: %~n0" /min "%BIN%winws.exe" --wf-tcp=80,443,%GameFilterTCP% ^
//! --filter-udp=443 --hostlist="%LISTS%list-general.txt" ... --new ^
//! ...
//! ```
//! We take everything after `winws.exe"`, join `^` continuations, substitute
//! the batch variables and split into argv the same way cmd would.

use crate::game_filter::GameFilter;
use crate::util::{find_ci, read_text, replace_ci};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyInfo {
    /// File name, e.g. `general (ALT3).bat`
    pub file: String,
    /// Display name, e.g. `ALT3` or `general`
    pub name: String,
}

fn natural_key(name: &str) -> String {
    // Same idea as service.bat: left-pad every number to 8 digits.
    let mut out = String::new();
    let mut num = String::new();
    for c in name.chars() {
        if c.is_ascii_digit() {
            num.push(c);
        } else {
            if !num.is_empty() {
                out.push_str(&format!("{:0>8}", num));
                num.clear();
            }
            out.push(c.to_ascii_lowercase());
        }
    }
    if !num.is_empty() {
        out.push_str(&format!("{:0>8}", num));
    }
    out
}

pub fn display_name(file: &str) -> String {
    let stem = file.trim_end_matches(".bat").trim_end_matches(".BAT");
    match (stem.find('('), stem.rfind(')')) {
        (Some(a), Some(b)) if b > a => stem[a + 1..b].trim().to_string(),
        _ => stem.to_string(),
    }
}

pub fn list(zapret: &Path) -> Vec<StrategyInfo> {
    let mut items: Vec<StrategyInfo> = fs::read_dir(zapret)
        .map(|rd| {
            rd.flatten()
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let lower = name.to_ascii_lowercase();
                    if lower.ends_with(".bat") && !lower.starts_with("service") && e.path().is_file() {
                        Some(StrategyInfo { name: display_name(&name), file: name })
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    // `general.bat` first, then natural order (ALT, ALT2 … ALT13, EXP, FAKE TLS …).
    items.sort_by_key(|s| (!s.file.eq_ignore_ascii_case("general.bat"), natural_key(&s.file)));
    items
}

/// Extracts the raw (unsubstituted) argument string after `winws.exe"`.
fn extract_raw(text: &str) -> Option<String> {
    let mut collected = String::new();
    let mut capturing = false;
    for line in text.lines() {
        let mut part = line.trim();
        if !capturing {
            match find_ci(part, "winws.exe") {
                Some(p) => {
                    capturing = true;
                    part = &part[p + "winws.exe".len()..];
                    part = part.strip_prefix('"').unwrap_or(part);
                }
                None => continue,
            }
        }
        let continued = part.ends_with('^');
        let body = part.trim_end_matches('^').trim();
        if !body.is_empty() {
            collected.push(' ');
            collected.push_str(body);
        }
        if !continued {
            break;
        }
    }
    if capturing {
        Some(collected)
    } else {
        None
    }
}

fn find_winws_line(text: &str) -> Option<String> {
    // Skip comment/echo lines that might mention winws.exe.
    let mut out = String::new();
    let mut started = false;
    for line in text.lines() {
        let t = line.trim_start();
        let lower = t.to_ascii_lowercase();
        if !started {
            if lower.starts_with("::") || lower.starts_with("rem ") || lower.starts_with("echo") {
                continue;
            }
            if !lower.contains("winws.exe") {
                continue;
            }
            started = true;
        }
        out.push_str(line);
        out.push('\n');
    }
    if started {
        extract_raw(&out)
    } else {
        None
    }
}

/// Splits a command line into arguments: whitespace separates, double quotes group
/// and are removed (cmd/MSVCRT semantics, sufficient for zapret strategies).
pub fn split_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut in_q = false;
    let mut has = false;
    for c in s.chars() {
        match c {
            '"' => {
                in_q = !in_q;
                has = true;
            }
            c if c.is_whitespace() && !in_q => {
                if has || !cur.is_empty() {
                    args.push(std::mem::take(&mut cur));
                    has = false;
                }
            }
            c => cur.push(c),
        }
    }
    if has || !cur.is_empty() {
        args.push(cur);
    }
    args
}

pub fn parse_args(zapret: &Path, file: &str, gf: &GameFilter) -> anyhow::Result<Vec<String>> {
    let path = zapret.join(file);
    if !path.starts_with(zapret) || file.contains("..") {
        anyhow::bail!("invalid strategy path");
    }
    let text = read_text(&path)?;
    let raw = find_winws_line(&text).ok_or_else(|| anyhow::anyhow!("winws.exe call not found in {file}"))?;

    let root = format!("{}\\", zapret.to_string_lossy().trim_end_matches(['\\', '/']));
    let bin = format!("{root}bin\\");
    let lists = format!("{root}lists\\");

    let mut s = raw;
    s = s.replace("^!", "!").replace("^^", "^");
    for (var, val) in [
        ("%BIN%", bin.clone()),
        ("%LISTS%", lists.clone()),
        ("%~dp0", root.clone()),
        ("%GameFilterTCP%", gf.tcp_value()),
        ("%GameFilterUDP%", gf.udp_value()),
        ("%GameFilter%", gf.combined_value()),
    ] {
        s = replace_ci(&s, var, &val);
    }
    let args = split_args(&s);
    if args.is_empty() {
        anyhow::bail!("no arguments parsed from {file}");
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample() {
        let bat = r#"@echo off
cd /d "%~dp0"
set "BIN=%~dp0bin\"
start "zapret: %~n0" /min "%BIN%winws.exe" --wf-tcp=80,443,%GameFilterTCP% --wf-udp=443,%GameFilterUDP% ^
--filter-udp=443 --hostlist="%LISTS%list-general.txt" --dpi-desync=fake --new ^
--filter-tcp=80 --dpi-desync-fake-tls="%BIN%tls clienthello.bin"
"#;
        let raw = find_winws_line(bat).unwrap();
        let gf = GameFilter::default();
        let mut s = raw;
        for (v, val) in [("%BIN%", "C:\\z\\bin\\".to_string()), ("%LISTS%", "C:\\z\\lists\\".into()), ("%GameFilterTCP%", gf.tcp_value()), ("%GameFilterUDP%", gf.udp_value())] {
            s = replace_ci(&s, v, &val);
        }
        let a = split_args(&s);
        assert_eq!(a[0], "--wf-tcp=80,443,12");
        assert_eq!(a[3], "--hostlist=C:\\z\\lists\\list-general.txt");
        assert_eq!(a.last().unwrap(), "--dpi-desync-fake-tls=C:\\z\\bin\\tls clienthello.bin");
        assert!(a.contains(&"--new".to_string()));
    }

    #[test]
    fn names() {
        assert_eq!(display_name("general (ALT11).bat"), "ALT11");
        assert_eq!(display_name("general.bat"), "general");
        assert!(natural_key("general (ALT2).bat") < natural_key("general (ALT10).bat"));
    }
}
