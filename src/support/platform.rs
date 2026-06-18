//! Shared environment and platform detection: shell, PATH membership, binary
//! lookup, and XDG base directories. Used by `setup` and the proxy shell flow
//! so detection logic lives in one place.

use std::path::{Path, PathBuf};

/// A user login shell that xrat knows how to integrate with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    /// Short lowercase name (`bash`, `zsh`, `fish`).
    pub fn name(self) -> &'static str {
        match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
        }
    }
}

/// Map a shell binary name or path to a [`Shell`]. Handles full paths and a
/// trailing `.exe`.
pub fn shell_from_name(name: &str) -> Option<Shell> {
    let base = name.rsplit('/').next().unwrap_or(name).trim();
    let base = base.strip_suffix(".exe").unwrap_or(base);
    match base {
        b if b.contains("fish") => Some(Shell::Fish),
        b if b.contains("zsh") => Some(Shell::Zsh),
        b if b.contains("bash") => Some(Shell::Bash),
        _ => None,
    }
}

/// Detect the active shell: `$SHELL` first, then the parent process name,
/// defaulting to bash.
pub fn detect_shell() -> Shell {
    if let Some(kind) = std::env::var("SHELL")
        .ok()
        .as_deref()
        .and_then(shell_from_name)
    {
        return kind;
    }
    if let Some(kind) = parent_process_name().as_deref().and_then(shell_from_name) {
        return kind;
    }
    Shell::Bash
}

fn parent_process_name() -> Option<String> {
    let ppid = sysinfo::Pid::from_u32(parent_process_id());
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[ppid]), false);
    system
        .process(ppid)
        .map(|process| process.name().to_string_lossy().to_string())
}

#[cfg(unix)]
fn parent_process_id() -> u32 {
    std::os::unix::process::parent_id()
}

#[cfg(windows)]
fn parent_process_id() -> u32 {
    std::os::windows::process::parent_id()
}

/// Resolve a binary name against `$PATH`, returning the first existing match.
pub fn binary_on_path(name: &str) -> Option<PathBuf> {
    binary_in(std::env::var_os("PATH")?.as_os_str(), name)
}

fn binary_in(path: &std::ffi::OsStr, name: &str) -> Option<PathBuf> {
    std::env::split_paths(path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

/// Whether `dir` is one of the entries in `$PATH`.
pub fn dir_in_path(dir: &Path) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    dir_in(path.as_os_str(), dir)
}

fn dir_in(path: &std::ffi::OsStr, dir: &Path) -> bool {
    std::env::split_paths(path).any(|entry| entry == dir)
}

/// Current OS name (`std::env::consts::OS`).
pub fn os() -> &'static str {
    std::env::consts::OS
}

/// Human-friendly OS name. On Linux this is the `PRETTY_NAME` from
/// `/etc/os-release` (e.g. "Fedora Linux 43"); otherwise the bare OS name.
pub fn os_pretty() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/etc/os-release")
            && let Some(name) = parse_os_release_pretty(&content)
        {
            return name;
        }
    }
    os().to_string()
}

#[cfg(any(target_os = "linux", test))]
fn parse_os_release_pretty(content: &str) -> Option<String> {
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
            let value = value.trim().trim_matches('"');
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Current CPU architecture (`std::env::consts::ARCH`).
pub fn arch() -> &'static str {
    std::env::consts::ARCH
}

/// `$HOME` as a path.
pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// `$XDG_DATA_HOME`, falling back to `$HOME/.local/share`.
pub fn xdg_data_home() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| home_dir().map(|home| home.join(".local").join("share")))
}

/// `$XDG_CONFIG_HOME`, falling back to `$HOME/.config`.
pub fn xdg_config_home() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| home_dir().map(|home| home.join(".config")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pretty_name_from_os_release() {
        let content = "NAME=\"Fedora Linux\"\nPRETTY_NAME=\"Fedora Linux 43 (Workstation Edition)\"\nID=fedora\n";
        assert_eq!(
            parse_os_release_pretty(content),
            Some("Fedora Linux 43 (Workstation Edition)".to_string())
        );
        assert_eq!(parse_os_release_pretty("ID=void\n"), None);
    }

    #[test]
    fn shell_from_name_handles_paths_and_suffixes() {
        assert_eq!(shell_from_name("/usr/bin/fish"), Some(Shell::Fish));
        assert_eq!(shell_from_name("/bin/zsh"), Some(Shell::Zsh));
        assert_eq!(shell_from_name("bash"), Some(Shell::Bash));
        assert_eq!(shell_from_name("C:\\bin\\bash.exe"), Some(Shell::Bash));
        assert_eq!(shell_from_name("nu"), None);
    }

    #[test]
    fn dir_in_detects_membership() {
        let dir = Path::new("/opt/xrat/bin");
        let other = Path::new("/usr/bin");
        let joined = std::env::join_paths([other, dir]).unwrap();
        assert!(dir_in(&joined, dir));
        assert!(!dir_in(&joined, Path::new("/definitely/not/on/path")));
    }

    #[test]
    fn binary_in_finds_existing_file() {
        let temp = std::env::temp_dir().join("xrat-platform-bin-test");
        std::fs::create_dir_all(&temp).unwrap();
        let binary = temp.join("xrat-fake-binary");
        std::fs::write(&binary, b"#!/bin/sh\n").unwrap();
        let joined = std::env::join_paths([temp.as_path()]).unwrap();
        assert_eq!(binary_in(&joined, "xrat-fake-binary"), Some(binary.clone()));
        assert_eq!(binary_in(&joined, "xrat-missing-binary"), None);
        std::fs::remove_file(&binary).ok();
    }
}
