//! Hostlist / IPSet editor backend. Comments and file structure are preserved:
//! entries are appended or removed line by line.

use crate::util::read_text;
use std::fs;
use std::path::{Path, PathBuf};

pub const EDITABLE: &[&str] = &[
    "list-general.txt",
    "list-general-user.txt",
    "list-exclude-user.txt",
    "list-google.txt",
    "ipset-all.txt",
    "ipset-exclude-user.txt",
];

fn path_for(z: &Path, name: &str) -> anyhow::Result<PathBuf> {
    if !EDITABLE.contains(&name) {
        anyhow::bail!("list is not editable: {name}");
    }
    Ok(z.join("lists").join(name))
}

fn is_entry(line: &str) -> bool {
    let t = line.trim();
    !t.is_empty() && !t.starts_with('#')
}

/// Placeholder used so a user list is never empty (an empty hostlist would match everything).
fn placeholder(name: &str) -> Option<&'static str> {
    match name {
        "list-general-user.txt" | "list-exclude-user.txt" => Some("domain.example.abc"),
        "ipset-exclude-user.txt" | "ipset-all.txt" => Some(crate::ipset::PLACEHOLDER),
        _ => None,
    }
}

/// Mirrors `service.bat :load_user_lists`.
pub fn ensure_user_lists(z: &Path) {
    let dir = z.join("lists");
    let _ = fs::create_dir_all(&dir);
    let files = [
        ("ipset-exclude-user.txt", "203.0.113.113/32\r\n"),
        ("list-general-user.txt", "# Never leave this file empty\r\ndomain.example.abc\r\n"),
        ("list-exclude-user.txt", "domain.example.abc\r\n"),
    ];
    for (name, content) in files {
        let p = dir.join(name);
        if !p.exists() {
            let _ = fs::write(p, content);
        }
    }
}

pub fn read_entries(z: &Path, name: &str) -> anyhow::Result<Vec<String>> {
    let p = path_for(z, name)?;
    let text = read_text(&p).unwrap_or_default();
    Ok(text.lines().filter(|l| is_entry(l)).map(|l| l.trim().to_string()).collect())
}

/// Normalises user input: strips scheme, path, port, `www.`-agnostic lowercase.
pub fn normalize(entry: &str, ipset: bool) -> Option<String> {
    let mut s = entry.trim().to_ascii_lowercase();
    if s.is_empty() || s.starts_with('#') {
        return None;
    }
    if !ipset {
        for scheme in ["https://", "http://"] {
            if let Some(rest) = s.strip_prefix(scheme) {
                s = rest.to_string();
            }
        }
        if let Some(i) = s.find(['/', '?', '#']) {
            s.truncate(i);
        }
        if let Some(i) = s.rfind(':') {
            if s[i + 1..].chars().all(|c| c.is_ascii_digit()) {
                s.truncate(i);
            }
        }
        let valid = s.contains('.')
            && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '^'));
        return valid.then_some(s);
    }
    let valid = s.chars().all(|c| c.is_ascii_hexdigit() || matches!(c, '.' | ':' | '/'))
        && (s.contains('.') || s.contains(':'));
    valid.then_some(s)
}

pub fn add(z: &Path, name: &str, entries: &[String]) -> anyhow::Result<Vec<String>> {
    let p = path_for(z, name)?;
    let ipset = name.starts_with("ipset");
    let mut text = read_text(&p).unwrap_or_default();
    let mut existing = read_entries(z, name)?;
    let ph = placeholder(name);

    let mut added = Vec::new();
    for e in entries {
        if let Some(n) = normalize(e, ipset) {
            if !existing.iter().any(|x| x.eq_ignore_ascii_case(&n)) && !added.contains(&n) {
                added.push(n);
            }
        }
    }
    if added.is_empty() {
        anyhow::bail!("NOTHING_ADDED");
    }
    // Drop the placeholder once real entries exist (ipset-all: switches "none" → "loaded").
    if let Some(ph) = ph {
        if existing.iter().any(|x| x == ph) {
            text = text.lines().filter(|l| l.trim() != ph).collect::<Vec<_>>().join("\r\n");
            existing.retain(|x| x != ph);
        }
    }
    if !text.is_empty() && !text.ends_with('\n') {
        text.push_str("\r\n");
    }
    for a in &added {
        text.push_str(a);
        text.push_str("\r\n");
    }
    fs::write(&p, text)?;
    read_entries(z, name)
}

pub fn remove(z: &Path, name: &str, entry: &str) -> anyhow::Result<Vec<String>> {
    let p = path_for(z, name)?;
    let text = read_text(&p).unwrap_or_default();
    let target = entry.trim();
    let kept: Vec<&str> = text.lines().filter(|l| !l.trim().eq_ignore_ascii_case(target)).collect();
    let mut out = kept.join("\r\n");
    if !out.is_empty() {
        out.push_str("\r\n");
    }
    let remaining = out.lines().filter(|l| is_entry(l)).count();
    if remaining == 0 {
        if let Some(ph) = placeholder(name) {
            out.push_str(ph);
            out.push_str("\r\n");
        }
    }
    fs::write(&p, out)?;
    read_entries(z, name)
}
