//! Shared helpers: console-less process spawning, filesystem utilities.

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// `CREATE_NO_WINDOW` — prevents any child process from opening a console window.
pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Creates a `Command` that never shows a console window.
pub fn hidden_cmd<S: AsRef<OsStr>>(program: S) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.stdin(Stdio::null());
    cmd
}

/// Runs a hidden command and captures its output.
pub fn run_hidden<I, S>(program: &str, args: I) -> io::Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    hidden_cmd(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
}

/// Recursively copies `src` into `dst`. `skip` receives the path relative to `src`
/// and returns `true` for entries that must not be copied.
pub fn copy_dir_filtered(src: &Path, dst: &Path, skip: &dyn Fn(&Path) -> bool) -> io::Result<()> {
    copy_inner(src, dst, Path::new(""), skip)
}

fn copy_inner(root: &Path, dst_root: &Path, rel: &Path, skip: &dyn Fn(&Path) -> bool) -> io::Result<()> {
    let src_dir = root.join(rel);
    let dst_dir = dst_root.join(rel);
    fs::create_dir_all(&dst_dir)?;
    for entry in fs::read_dir(&src_dir)? {
        let entry = entry?;
        let rel_path = rel.join(entry.file_name());
        if skip(&rel_path) {
            continue;
        }
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_inner(root, dst_root, &rel_path, skip)?;
        } else {
            let target = dst_root.join(&rel_path);
            // Clear read-only attribute on an existing target so it can be replaced.
            if let Ok(meta) = fs::metadata(&target) {
                let mut perms = meta.permissions();
                if perms.readonly() {
                    #[allow(clippy::permissions_set_readonly_false)]
                    perms.set_readonly(false);
                    let _ = fs::set_permissions(&target, perms);
                }
            }
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    copy_dir_filtered(src, dst, &|_| false)
}

/// Total size of a directory tree in bytes.
pub fn dir_size(path: &Path) -> u64 {
    let mut total = 0;
    if let Ok(rd) = fs::read_dir(path) {
        for e in rd.flatten() {
            match e.file_type() {
                Ok(t) if t.is_dir() => total += dir_size(&e.path()),
                Ok(_) => total += e.metadata().map(|m| m.len()).unwrap_or(0),
                Err(_) => {}
            }
        }
    }
    total
}

/// Reads a text file leniently (UTF-8 with lossy fallback, BOM stripped).
pub fn read_text(path: &Path) -> io::Result<String> {
    let bytes = fs::read(path)?;
    Ok(String::from_utf8_lossy(&bytes).trim_start_matches('\u{feff}').to_string())
}

/// Case-insensitive ASCII search; returns the byte offset of `needle` in `hay`.
pub fn find_ci(hay: &str, needle: &str) -> Option<usize> {
    hay.to_ascii_lowercase().find(&needle.to_ascii_lowercase())
}

/// Case-insensitive ASCII replace of every occurrence of `from` with `to`.
pub fn replace_ci(input: &str, from: &str, to: &str) -> String {
    let lower = input.to_ascii_lowercase();
    let from_l = from.to_ascii_lowercase();
    let mut out = String::with_capacity(input.len());
    let mut last = 0;
    let mut search = 0;
    while let Some(pos) = lower[search..].find(&from_l) {
        let start = search + pos;
        out.push_str(&input[last..start]);
        out.push_str(to);
        last = start + from.len();
        search = last;
    }
    out.push_str(&input[last..]);
    out
}

/// Compares dotted version strings numerically ("1.10.4" > "1.9.7").
pub fn version_newer(latest: &str, local: &str) -> bool {
    let parse = |s: &str| -> Vec<u64> {
        s.trim()
            .trim_start_matches(['v', 'V'])
            .split(|c: char| !c.is_ascii_digit())
            .filter(|p| !p.is_empty())
            .map(|p| p.parse().unwrap_or(0))
            .collect()
    };
    let (a, b) = (parse(latest), parse(local));
    for i in 0..a.len().max(b.len()) {
        let (x, y) = (a.get(i).copied().unwrap_or(0), b.get(i).copied().unwrap_or(0));
        if x != y {
            return x > y;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions() {
        assert!(version_newer("1.10.4", "1.9.9"));
        assert!(!version_newer("1.10.4", "1.10.4"));
        assert!(version_newer("v2", "1.99"));
    }

    #[test]
    fn replace() {
        assert_eq!(replace_ci("a %BIN%x %bin%", "%BIN%", "Q"), "a Qx Q");
    }
}
