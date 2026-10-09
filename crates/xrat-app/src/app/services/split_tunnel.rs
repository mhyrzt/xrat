use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrat_engines::singbox::{SingboxTunSplitMode, SingboxTunSplitOptions};
use xrat_engines::xray::{XrayTunSplitMode, XrayTunSplitOptions};

use crate::app::config::{TunSettings, TunSplitMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AppSource {
    Running,
    Desktop,
    RunningAndDesktop,
}

impl AppSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Desktop => "desktop",
            Self::RunningAndDesktop => "running+desktop",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredApp {
    pub name: String,
    pub desktop_id: Option<String>,
    pub process_name: String,
    pub exec_path: Option<String>,
    pub rule_entry: String,
    pub source: AppSource,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledSplitRules {
    pub mode: TunSplitMode,
    pub xray_processes: Vec<String>,
    pub singbox_process_names: Vec<String>,
    pub singbox_process_paths: Vec<String>,
    pub singbox_process_path_regexes: Vec<String>,
    pub warnings: Vec<String>,
}

impl CompiledSplitRules {
    pub fn to_xray_options(&self) -> XrayTunSplitOptions {
        let mode = match self.mode {
            TunSplitMode::All => XrayTunSplitMode::All,
            TunSplitMode::Blacklist => XrayTunSplitMode::Blacklist,
            TunSplitMode::Whitelist => XrayTunSplitMode::Whitelist,
        };
        XrayTunSplitOptions {
            mode,
            processes: self.xray_processes.clone(),
        }
    }

    pub fn to_singbox_options(&self) -> SingboxTunSplitOptions {
        let mode = match self.mode {
            TunSplitMode::All => SingboxTunSplitMode::All,
            TunSplitMode::Blacklist => SingboxTunSplitMode::Blacklist,
            TunSplitMode::Whitelist => SingboxTunSplitMode::Whitelist,
        };
        SingboxTunSplitOptions {
            mode,
            process_name: self.singbox_process_names.clone(),
            process_path: self.singbox_process_paths.clone(),
            process_path_regex: self.singbox_process_path_regexes.clone(),
        }
    }
}

pub fn compile_split_rules(tun: &TunSettings) -> CompiledSplitRules {
    compile_split_rules_with_dirs(tun, &desktop_search_dirs())
}

pub fn compile_split_rules_with_dirs(
    tun: &TunSettings,
    desktop_dirs: &[PathBuf],
) -> CompiledSplitRules {
    let entries = match tun.split_mode {
        TunSplitMode::All => &[][..],
        TunSplitMode::Blacklist => tun.blacklist.as_slice(),
        TunSplitMode::Whitelist => tun.whitelist.as_slice(),
    };

    let mut xray_processes = Vec::new();
    let mut singbox_process_names = Vec::new();
    let mut singbox_process_paths = Vec::new();
    let mut singbox_process_path_regexes = Vec::new();
    let mut warnings = Vec::new();
    let mut seen = BTreeSet::new();

    for raw in entries {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let resolved_items = if trimmed.starts_with("desktop:") || trimmed.ends_with(".desktop") {
            match resolve_desktop_specifier_in_dirs(trimmed, desktop_dirs) {
                Some(items) => items,
                None => {
                    let fallback = trimmed
                        .strip_prefix("desktop:")
                        .unwrap_or(trimmed)
                        .strip_suffix(".desktop")
                        .unwrap_or(trimmed)
                        .trim();
                    warnings.push(format!(
                        "desktop application `{trimmed}` was not found in XDG desktop directories; falling back to `{fallback}`"
                    ));
                    if fallback.is_empty() {
                        Vec::new()
                    } else {
                        vec![fallback.to_string()]
                    }
                }
            }
        } else {
            vec![trimmed.to_string()]
        };

        for item in resolved_items {
            if !seen.insert(item.clone()) {
                continue;
            }
            if item.starts_with('/') && item.ends_with('/') {
                xray_processes.push(item.clone());
                singbox_process_path_regexes.push(format!("^{}.*", escape_regex(&item)));
            } else if item.starts_with('/') {
                xray_processes.push(item.clone());
                singbox_process_paths.push(item);
            } else {
                xray_processes.push(item.clone());
                singbox_process_names.push(item);
            }
        }
    }

    CompiledSplitRules {
        mode: tun.split_mode,
        xray_processes,
        singbox_process_names,
        singbox_process_paths,
        singbox_process_path_regexes,
        warnings,
    }
}

/// Validate and normalize a user-supplied application specifier before adding it
/// to `blacklist` or `whitelist`. When `resolve_desktop` is true, `.desktop`
/// files or matching desktop application IDs are resolved to their binary name.
pub fn normalize_app_entry(raw: &str, resolve_desktop: bool) -> Result<Vec<String>, String> {
    normalize_app_entry_with_dirs(raw, resolve_desktop, &desktop_search_dirs())
}

pub fn normalize_app_specifier(raw: &str) -> Result<Vec<String>, String> {
    normalize_app_entry(raw, true)
}

pub fn normalize_app_specifier_with_dirs(
    raw: &str,
    desktop_dirs: &[PathBuf],
) -> Result<Vec<String>, String> {
    normalize_app_entry_with_dirs(raw, true, desktop_dirs)
}

pub fn normalize_app_entry_with_dirs(
    raw: &str,
    resolve_desktop: bool,
    desktop_dirs: &[PathBuf],
) -> Result<Vec<String>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("application entry cannot be empty".to_string());
    }
    if trimmed.contains('\0') || trimmed.contains('\n') || trimmed.contains('\r') {
        return Err("application entry cannot contain newline or NUL characters".to_string());
    }

    if resolve_desktop {
        if let Some(resolved) = resolve_desktop_specifier_in_dirs(trimmed, desktop_dirs)
            && !resolved.is_empty()
        {
            return Ok(resolved);
        }
        if trimmed.starts_with("desktop:") || trimmed.ends_with(".desktop") {
            return Err(format!(
                "could not find or resolve desktop application {trimmed:?} in XDG application directories"
            ));
        }
    }

    if trimmed.contains('/') && !trimmed.starts_with('/') && !trimmed.starts_with("desktop:") {
        return Err(format!(
            "relative path {trimmed:?} is not allowed; use a bare process name, an absolute path (/usr/bin/app), or a directory (/opt/app/)"
        ));
    }

    Ok(vec![trimmed.to_string()])
}

pub fn desktop_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(data_home) = std::env::var_os("XDG_DATA_HOME") {
        dirs.push(PathBuf::from(data_home).join("applications"));
    } else if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share/applications"));
    }

    if let Some(data_dirs) = std::env::var_os("XDG_DATA_DIRS") {
        for dir in std::env::split_paths(&data_dirs) {
            dirs.push(dir.join("applications"));
        }
    } else {
        dirs.push(PathBuf::from("/usr/local/share/applications"));
        dirs.push(PathBuf::from("/usr/share/applications"));
    }

    for extra in [
        "/var/lib/flatpak/exports/share/applications",
        "/var/lib/snapd/desktop/applications",
    ] {
        let path = PathBuf::from(extra);
        if !dirs.contains(&path) {
            dirs.push(path);
        }
    }
    dirs
}

pub fn resolve_desktop_specifier_in_dirs(
    specifier: &str,
    desktop_dirs: &[PathBuf],
) -> Option<Vec<String>> {
    let query = specifier
        .strip_prefix("desktop:")
        .unwrap_or(specifier)
        .trim();
    if query.is_empty() {
        return None;
    }

    // If the user passed an explicit path to a .desktop file
    let direct_path = Path::new(query);
    if direct_path.is_absolute() && direct_path.extension().is_some_and(|ext| ext == "desktop") {
        let content = std::fs::read_to_string(direct_path).ok()?;
        let id = direct_path.file_name()?.to_string_lossy().into_owned();
        let (_, exec_target) = parse_desktop_entry_content(&id, &content)?;
        return Some(resolve_exec_target_rules(&exec_target));
    }

    let query_with_ext = if query.ends_with(".desktop") {
        query.to_string()
    } else {
        format!("{query}.desktop")
    };
    let query_lower = query.to_ascii_lowercase();
    let query_with_ext_lower = query_with_ext.to_ascii_lowercase();

    // First pass: exact filename match (`firefox.desktop` or `org.mozilla.firefox.desktop`)
    for dir in desktop_dirs {
        let candidate = dir.join(&query_with_ext);
        if candidate.is_file()
            && let Ok(content) = std::fs::read_to_string(&candidate)
            && let Some((_, exec_target)) = parse_desktop_entry_content(&query_with_ext, &content)
        {
            return Some(resolve_exec_target_rules(&exec_target));
        }
    }

    // Second pass: case-insensitive desktop ID suffix or Name match only when explicitly requested (`desktop:` or `.desktop`)
    let explicit_desktop =
        specifier.starts_with("desktop:") || specifier.ends_with(".desktop") || query.contains('.');
    if !explicit_desktop {
        return None;
    }

    for app in discover_desktop_apps_in_dirs(desktop_dirs) {
        let id_matches = app.desktop_id.as_ref().is_some_and(|id| {
            let id_lower = id.to_ascii_lowercase();
            id_lower == query_with_ext_lower
                || id_lower == query_lower
                || id_lower
                    .strip_suffix(".desktop")
                    .is_some_and(|stem| stem.ends_with(&format!(".{query_lower}")))
        });
        let name_matches = app.name.to_ascii_lowercase() == query_lower;
        if id_matches || name_matches {
            return Some(vec![app.process_name]);
        }
    }

    None
}

fn resolve_exec_target_rules(exec_target: &str) -> Vec<String> {
    let mut results = Vec::new();
    let path = Path::new(exec_target);
    if let Some(name) = path.file_name().and_then(|n| n.to_str())
        && !name.is_empty()
    {
        results.push(name.to_string());
    }
    let resolved_path = if path.is_absolute() {
        Some(path.to_path_buf())
    } else {
        crate::app::tun_privileges::resolve_executable(path)
    };
    if let Some(full_path) = resolved_path
        && let Ok(canonical) = std::fs::canonicalize(&full_path)
        && let Some(canon_name) = canonical.file_name().and_then(|n| n.to_str())
        && !canon_name.is_empty()
        && !matches!(
            canon_name,
            "sh" | "bash" | "dash" | "zsh" | "python" | "python3" | "env" | "flatpak" | "snap"
        )
        && !results.iter().any(|existing| existing == canon_name)
    {
        results.push(canon_name.to_string());
    }
    if results.is_empty() {
        results.push(exec_target.to_string());
    }
    results
}

pub fn parse_desktop_entry_content(desktop_id: &str, content: &str) -> Option<(String, String)> {
    let mut in_desktop_entry = false;
    let mut name: Option<String> = None;
    let mut try_exec: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut entry_type: Option<String> = None;
    let mut no_display = false;
    let mut hidden = false;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            if line == "[Desktop Entry]" {
                in_desktop_entry = true;
                continue;
            } else if in_desktop_entry {
                break;
            } else {
                continue;
            }
        }
        if !in_desktop_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "Name" if name.is_none() => {
                if !value.is_empty() {
                    name = Some(value.to_string());
                }
            }
            "TryExec" if try_exec.is_none() => {
                if !value.is_empty() {
                    try_exec = Some(value.to_string());
                }
            }
            "Exec" if exec.is_none() => {
                if !value.is_empty() {
                    exec = Some(value.to_string());
                }
            }
            "Type" => {
                entry_type = Some(value.to_string());
            }
            "NoDisplay" => {
                no_display = value.eq_ignore_ascii_case("true");
            }
            "Hidden" => {
                hidden = value.eq_ignore_ascii_case("true");
            }
            _ => {}
        }
    }

    if no_display || hidden {
        return None;
    }
    if let Some(kind) = &entry_type
        && kind != "Application"
    {
        return None;
    }

    let exec_target = exec
        .as_deref()
        .and_then(extract_executable_from_exec_line)
        .or_else(|| {
            try_exec
                .as_deref()
                .and_then(extract_executable_from_exec_line)
        })?;
    let display_name = name.unwrap_or_else(|| {
        desktop_id
            .strip_suffix(".desktop")
            .unwrap_or(desktop_id)
            .to_string()
    });
    Some((display_name, exec_target))
}

pub fn extract_executable_from_exec_line(exec: &str) -> Option<String> {
    let tokens = tokenize_desktop_exec(exec);
    if tokens.is_empty() {
        return None;
    }

    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        let base = Path::new(token)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(token);
        if base == "env" {
            index += 1;
            while index < tokens.len() {
                let next = &tokens[index];
                if next.starts_with('-') || (next.contains('=') && !next.starts_with('/')) {
                    index += 1;
                } else {
                    break;
                }
            }
            continue;
        }
        if token.contains('=') && !token.starts_with('/') {
            index += 1;
            continue;
        }
        if base == "flatpak" {
            for flatpak_arg in &tokens[index + 1..] {
                if let Some(cmd) = flatpak_arg.strip_prefix("--command=")
                    && !cmd.is_empty()
                {
                    return Some(cmd.to_string());
                }
            }
            // Fallback to the application ID after `flatpak run`
            let mut after_run = false;
            for flatpak_arg in &tokens[index + 1..] {
                if flatpak_arg == "run" {
                    after_run = true;
                    continue;
                }
                if after_run
                    && !flatpak_arg.starts_with('-')
                    && !flatpak_arg.starts_with('%')
                    && !flatpak_arg.starts_with("@@")
                {
                    return Some(flatpak_arg.clone());
                }
            }
        }
        if matches!(base, "sh" | "bash" | "dash" | "zsh")
            && tokens.get(index + 1).is_some_and(|arg| arg == "-c")
            && let Some(inner) = tokens.get(index + 2)
        {
            return extract_executable_from_exec_line(inner);
        }
        if token.starts_with('%') {
            index += 1;
            continue;
        }
        return Some(token.clone());
    }
    None
}

fn tokenize_desktop_exec(exec: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut chars = exec.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' if !in_single => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            c if c.is_whitespace() && !in_single && !in_double => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

pub fn discover_apps(
    include_running: bool,
    include_desktop: bool,
    query: Option<&str>,
) -> Vec<DiscoveredApp> {
    discover_apps_with_dirs(
        include_running,
        include_desktop,
        query,
        &desktop_search_dirs(),
    )
}

pub fn discover_apps_with_dirs(
    include_running: bool,
    include_desktop: bool,
    query: Option<&str>,
    desktop_dirs: &[PathBuf],
) -> Vec<DiscoveredApp> {
    let mut by_process: BTreeMap<String, DiscoveredApp> = BTreeMap::new();

    if include_running {
        for app in discover_running_apps() {
            by_process.insert(app.process_name.clone(), app);
        }
    }

    if include_desktop {
        for app in discover_desktop_apps_in_dirs(desktop_dirs) {
            match by_process.get_mut(&app.process_name) {
                Some(existing) => {
                    existing.name = app.name;
                    existing.desktop_id = app.desktop_id;
                    if existing.exec_path.is_none() {
                        existing.exec_path = app.exec_path;
                    }
                    existing.source = AppSource::RunningAndDesktop;
                }
                None => {
                    by_process.insert(app.process_name.clone(), app);
                }
            }
        }
    }

    let query_lower = query
        .map(str::trim)
        .filter(|q| !q.is_empty())
        .map(str::to_ascii_lowercase);

    let mut apps: Vec<DiscoveredApp> = by_process
        .into_values()
        .filter(|app| {
            let Some(q) = &query_lower else {
                return true;
            };
            app.name.to_ascii_lowercase().contains(q)
                || app.process_name.to_ascii_lowercase().contains(q)
                || app
                    .desktop_id
                    .as_ref()
                    .is_some_and(|id| id.to_ascii_lowercase().contains(q))
                || app
                    .exec_path
                    .as_ref()
                    .is_some_and(|path| path.to_ascii_lowercase().contains(q))
        })
        .collect();

    apps.sort_by(|a, b| {
        let a_running = matches!(a.source, AppSource::Running | AppSource::RunningAndDesktop);
        let b_running = matches!(b.source, AppSource::Running | AppSource::RunningAndDesktop);
        b_running
            .cmp(&a_running)
            .then_with(|| a.process_name.cmp(&b.process_name))
    });
    apps
}

pub fn discover_desktop_apps_in_dirs(desktop_dirs: &[PathBuf]) -> Vec<DiscoveredApp> {
    let mut seen_ids = BTreeSet::new();
    let mut apps = Vec::new();

    for dir in desktop_dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() || path.extension().is_none_or(|ext| ext != "desktop") {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !seen_ids.insert(file_name.to_string()) {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Some((name, exec_target)) = parse_desktop_entry_content(file_name, &content) else {
                continue;
            };
            let exec_path_obj = Path::new(&exec_target);
            let process_name = exec_path_obj
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&exec_target)
                .to_string();
            if process_name.is_empty() {
                continue;
            }
            let exec_path = if exec_path_obj.is_absolute() {
                Some(exec_target)
            } else {
                crate::app::tun_privileges::resolve_executable(exec_path_obj)
                    .map(|p| p.display().to_string())
            };
            apps.push(DiscoveredApp {
                name,
                desktop_id: Some(file_name.to_string()),
                rule_entry: process_name.clone(),
                process_name,
                exec_path,
                source: AppSource::Desktop,
            });
        }
    }
    apps
}

pub fn discover_running_apps() -> Vec<DiscoveredApp> {
    #[cfg(target_os = "linux")]
    {
        let mut by_name: BTreeMap<String, DiscoveredApp> = BTreeMap::new();
        let Ok(entries) = std::fs::read_dir("/proc") else {
            return Vec::new();
        };
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Some(pid_str) = file_name.to_str() else {
                continue;
            };
            if !pid_str.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let proc_dir = entry.path();
            let exe_path = std::fs::read_link(proc_dir.join("exe")).ok().map(|path| {
                let display = path.display().to_string();
                display
                    .strip_suffix(" (deleted)")
                    .unwrap_or(&display)
                    .to_string()
            });
            let process_name = exe_path
                .as_deref()
                .and_then(|p| Path::new(p).file_name())
                .and_then(|n| n.to_str())
                .map(str::to_string)
                .or_else(|| {
                    std::fs::read_to_string(proc_dir.join("comm"))
                        .ok()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty() && !s.starts_with('['))
                });
            let Some(process_name) = process_name else {
                continue;
            };
            if matches!(
                process_name.as_str(),
                "xrat" | "xray" | "sing-box" | "v2ray" | "systemd" | "init" | "kthreadd"
            ) {
                continue;
            }
            by_name
                .entry(process_name.clone())
                .or_insert_with(|| DiscoveredApp {
                    name: process_name.clone(),
                    desktop_id: None,
                    rule_entry: process_name.clone(),
                    process_name,
                    exec_path: exe_path,
                    source: AppSource::Running,
                });
        }
        by_name.into_values().collect()
    }
    #[cfg(not(target_os = "linux"))]
    {
        Vec::new()
    }
}

fn escape_regex(literal: &str) -> String {
    let mut escaped = String::with_capacity(literal.len());
    for ch in literal.chars() {
        if matches!(
            ch,
            '.' | '+' | '*' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\'
        ) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_executable_from_complex_desktop_exec_lines() {
        assert_eq!(
            extract_executable_from_exec_line("/usr/lib/firefox/firefox %u"),
            Some("/usr/lib/firefox/firefox".to_string())
        );
        assert_eq!(
            extract_executable_from_exec_line(
                "env BAMF_DESKTOP_FILE_HINT=/var/lib/snapd/desktop/applications/telegram.desktop /opt/telegram/Telegram -- %u"
            ),
            Some("/opt/telegram/Telegram".to_string())
        );
        assert_eq!(
            extract_executable_from_exec_line(
                "/usr/bin/flatpak run --branch=stable --arch=x86_64 --command=telegram-desktop org.telegram.desktop -- %u"
            ),
            Some("telegram-desktop".to_string())
        );
        assert_eq!(
            extract_executable_from_exec_line("\"/opt/Google Chrome/chrome\" --new-window %U"),
            Some("/opt/Google Chrome/chrome".to_string())
        );
    }

    #[test]
    fn parses_desktop_files_and_skips_hidden_or_non_app_entries() {
        let valid = r#"
[Desktop Entry]
Type=Application
Name=Firefox Web Browser
Exec=/usr/lib/firefox/firefox %u
"#;
        assert_eq!(
            parse_desktop_entry_content("firefox.desktop", valid),
            Some((
                "Firefox Web Browser".to_string(),
                "/usr/lib/firefox/firefox".to_string()
            ))
        );

        let hidden = r#"
[Desktop Entry]
Type=Application
Name=Hidden Helper
NoDisplay=true
Exec=/usr/bin/helper
"#;
        assert_eq!(parse_desktop_entry_content("helper.desktop", hidden), None);
    }

    #[test]
    fn resolves_desktop_specifiers_from_custom_directory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("org.telegram.desktop"),
            "[Desktop Entry]\nType=Application\nName=Telegram Desktop\nExec=/usr/bin/telegram-desktop -- %u\n",
        )
        .unwrap();
        let dirs = vec![dir.path().to_path_buf()];

        let resolved = normalize_app_entry_with_dirs("org.telegram.desktop", true, &dirs).unwrap();
        assert_eq!(resolved, vec!["telegram-desktop".to_string()]);

        let by_name =
            normalize_app_entry_with_dirs("desktop:Telegram Desktop", true, &dirs).unwrap();
        assert_eq!(by_name, vec!["telegram-desktop".to_string()]);
    }

    #[test]
    fn compiles_split_rules_for_xray_and_singbox() {
        let tun = TunSettings {
            split_mode: TunSplitMode::Blacklist,
            blacklist: vec![
                "firefox".to_string(),
                "/usr/bin/curl".to_string(),
                "/opt/discord/".to_string(),
            ],
            ..Default::default()
        };

        let compiled = compile_split_rules_with_dirs(&tun, &[]);
        assert_eq!(compiled.mode, TunSplitMode::Blacklist);
        assert_eq!(
            compiled.xray_processes,
            vec!["firefox", "/usr/bin/curl", "/opt/discord/"]
        );
        assert_eq!(compiled.singbox_process_names, vec!["firefox"]);
        assert_eq!(compiled.singbox_process_paths, vec!["/usr/bin/curl"]);
        assert_eq!(
            compiled.singbox_process_path_regexes,
            vec!["^/opt/discord/.*"]
        );
    }
}
