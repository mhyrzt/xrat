use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;
use toml_edit::{Array, DocumentMut, InlineTable, Item, Table, Value, value as toml_value};

use super::AppConfig;
use help::SettingHelp;

mod help;

const SECRET_PATHS: &[&str] = &[
    "runtime.socks.auth.username",
    "runtime.socks.auth.password",
    "runtime.shadowsocks.password",
    "testing.geoip.remote.api_key",
    "server.key",
];
const NUMERIC_LIST_PATHS: &[&str] = &[
    "runtime.fragment.packets",
    "runtime.fragment.length",
    "runtime.fragment.interval",
    "testing.real_delay.accepted_status_codes",
];
const OPTIONAL_LIST_PATHS: &[&str] = &[
    "testing.real_delay.accepted_status_codes",
    "testing.real_delay.accepted_status_ranges",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SettingEffect {
    Live,
    RuntimeRestart,
    DaemonRestart,
}

impl SettingEffect {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::RuntimeRestart => "runtime restart",
            Self::DaemonRestart => "daemon restart",
        }
    }

    pub(crate) fn help_text(self) -> &'static str {
        match self {
            Self::Live => "Applies immediately to subsequent TUI operations.",
            Self::RuntimeRestart => "Requires restarting the active proxy runtime.",
            Self::DaemonRestart => "Requires restarting the xrat daemon.",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SettingKind {
    Bool,
    Integer,
    Text,
    List { numeric: bool },
    Enum(&'static [&'static str]),
    Secret,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SettingValue {
    Bool(bool),
    Integer(i64),
    Text(String),
    List(Vec<String>),
    Secret(String),
}

impl SettingValue {
    pub(crate) fn display(&self, secret: bool) -> String {
        if secret {
            return if matches!(self, Self::Secret(value) if value.is_empty()) {
                "not set".to_string()
            } else {
                "•••• configured".to_string()
            };
        }
        match self {
            Self::Bool(value) => value.to_string(),
            Self::Integer(value) => value.to_string(),
            Self::Text(value) => {
                if value.is_empty() {
                    "<empty>".to_string()
                } else {
                    value.clone()
                }
            }
            Self::List(values) => {
                if values.is_empty() {
                    "[]".to_string()
                } else {
                    values.join(", ")
                }
            }
            Self::Secret(_) => "•••• configured".to_string(),
        }
    }

    pub(crate) fn edit_text(&self) -> String {
        match self {
            Self::Bool(value) => value.to_string(),
            Self::Integer(value) => value.to_string(),
            Self::Text(value) | Self::Secret(value) => value.clone(),
            Self::List(values) => values.join(", "),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EditableSetting {
    pub(crate) path: String,
    pub(crate) section: String,
    pub(crate) label: String,
    pub(crate) kind: SettingKind,
    pub(crate) effect: SettingEffect,
    pub(crate) help: SettingHelp,
    pub(crate) value: SettingValue,
    default_value: SettingValue,
    original_value: SettingValue,
    explicit: bool,
    reset: bool,
}

impl EditableSetting {
    pub(crate) fn is_dirty(&self) -> bool {
        self.value != self.original_value || (self.reset && self.explicit)
    }

    pub(crate) fn is_explicit(&self) -> bool {
        self.explicit
    }

    pub(crate) fn is_reset(&self) -> bool {
        self.reset
    }

    pub(crate) fn default_value(&self) -> &SettingValue {
        &self.default_value
    }

    pub(crate) fn possible_values(&self) -> String {
        match &self.kind {
            SettingKind::Bool => "✓ enabled · ✗ disabled".to_string(),
            SettingKind::Enum(options) => options.join(" · "),
            _ => self.help.value_hint.to_string(),
        }
    }

    pub(crate) fn toggle(&mut self) -> bool {
        let SettingValue::Bool(value) = &mut self.value else {
            return false;
        };
        *value = !*value;
        self.reset = false;
        true
    }

    pub(crate) fn cycle_enum(&mut self, direction: i32) -> bool {
        let SettingKind::Enum(options) = &self.kind else {
            return false;
        };
        let SettingValue::Text(current) = &mut self.value else {
            return false;
        };
        let position = options
            .iter()
            .position(|option| *option == current)
            .unwrap_or(0);
        let next = if direction < 0 {
            (position + options.len() - 1) % options.len()
        } else {
            (position + 1) % options.len()
        };
        *current = options[next].to_string();
        self.reset = false;
        true
    }

    pub(crate) fn set_from_input(&mut self, input: &str) -> Result<(), String> {
        self.value = match self.kind {
            SettingKind::Integer => SettingValue::Integer(
                input
                    .trim()
                    .parse()
                    .map_err(|_| "enter a whole number".to_string())?,
            ),
            SettingKind::Text => SettingValue::Text(input.trim().to_string()),
            SettingKind::List { numeric } => {
                let values: Vec<String> = input
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
                    .collect();
                if numeric && values.iter().any(|value| value.parse::<i64>().is_err()) {
                    return Err("list values must be whole numbers".to_string());
                }
                SettingValue::List(values)
            }
            SettingKind::Secret => SettingValue::Secret(input.trim().to_string()),
            SettingKind::Bool | SettingKind::Enum(_) => {
                return Err("use the toggle or cycle keys for this setting".to_string());
            }
        };
        self.reset = false;
        Ok(())
    }

    pub(crate) fn reset_to_default(&mut self) {
        self.value = self.default_value.clone();
        self.reset = true;
    }
}

#[derive(Debug)]
pub(crate) struct ConfigEditSession {
    path: PathBuf,
    original_contents: String,
    document: DocumentMut,
    pub(crate) settings: Vec<EditableSetting>,
}

#[derive(Debug)]
pub(crate) struct ConfigSaveOutcome {
    pub(crate) config: AppConfig,
    pub(crate) changed_paths: Vec<String>,
    pub(crate) effects: BTreeSet<SettingEffect>,
}

impl ConfigEditSession {
    pub(crate) fn open(path: &Path) -> Result<Self, String> {
        let original_contents = fs::read_to_string(path)
            .map_err(|error| format!("could not read {}: {error}", path.display()))?;
        let document = original_contents
            .parse::<DocumentMut>()
            .map_err(|error| format!("config is not valid TOML: {error}"))?;
        let config: AppConfig = toml::from_str(&original_contents)
            .map_err(|error| format!("config could not be loaded: {error}"))?;
        let defaults = operational_values(&AppConfig::default())?;
        let current = operational_values(&config)?;
        let mut settings = Vec::new();
        flatten_settings("", &current, &defaults, &document, &mut settings);
        settings
            .sort_by(|left, right| (&left.section, &left.path).cmp(&(&right.section, &right.path)));
        Ok(Self {
            path: path.to_path_buf(),
            original_contents,
            document,
            settings,
        })
    }

    pub(crate) fn is_dirty(&self) -> bool {
        self.settings.iter().any(EditableSetting::is_dirty)
    }

    pub(crate) fn path_display(&self) -> String {
        self.path.display().to_string()
    }

    pub(crate) fn sections(&self, query: &str) -> Vec<String> {
        let query = query.trim().to_ascii_lowercase();
        let mut sections = BTreeSet::new();
        for setting in &self.settings {
            if query.is_empty()
                || setting.path.to_ascii_lowercase().contains(&query)
                || setting.label.to_ascii_lowercase().contains(&query)
            {
                sections.insert(setting.section.clone());
            }
        }
        sections.into_iter().collect()
    }

    pub(crate) fn set_value(&mut self, path: &str, input: &str) -> Result<(), String> {
        let setting = self
            .settings
            .iter_mut()
            .find(|setting| setting.path == path)
            .ok_or_else(|| format!("setting {path:?} was not found"))?;
        setting.set_from_input(input)
    }

    pub(crate) fn save(&mut self) -> Result<ConfigSaveOutcome, String> {
        let current_contents = fs::read_to_string(&self.path)
            .map_err(|error| format!("could not re-read {}: {error}", self.path.display()))?;
        if current_contents != self.original_contents {
            return Err(
                "config changed on disk; close and reopen settings before saving".to_string(),
            );
        }
        if !self.is_dirty() {
            let config = toml::from_str(&current_contents)
                .map_err(|error| format!("current config is invalid: {error}"))?;
            return Ok(ConfigSaveOutcome {
                config,
                changed_paths: Vec::new(),
                effects: BTreeSet::new(),
            });
        }

        let mut document = self.document.clone();
        let mut changed_paths = Vec::new();
        let mut effects = BTreeSet::new();
        for setting in self.settings.iter().filter(|setting| setting.is_dirty()) {
            if setting.reset {
                remove_path(&mut document, &setting.path);
            } else {
                set_path(&mut document, &setting.path, setting_item(setting)?);
            }
            changed_paths.push(setting.path.clone());
            effects.insert(setting.effect);
        }

        let candidate = document.to_string();
        let config: AppConfig = toml::from_str(&candidate)
            .map_err(|error| format!("updated config is invalid: {error}"))?;
        let diagnostics = crate::app::commands::validate::validate_app_config(&config);
        if let Some(diagnostic) = diagnostics.first() {
            return Err(diagnostic.clone());
        }
        atomic_write(&self.path, candidate.as_bytes())?;

        self.original_contents = candidate;
        self.document = document;
        for setting in &mut self.settings {
            setting.original_value = setting.value.clone();
            setting.explicit = path_exists(&self.document, &setting.path);
            setting.reset = false;
        }

        Ok(ConfigSaveOutcome {
            config,
            changed_paths,
            effects,
        })
    }
}

pub(crate) fn update_runtime_binary_path(
    config_path: &Path,
    key: &str,
    binary_path: &Path,
) -> Result<(), String> {
    let value = binary_path
        .to_str()
        .ok_or_else(|| "managed binary path is not valid UTF-8".to_string())?;
    let contents = fs::read_to_string(config_path)
        .map_err(|error| format!("could not read {}: {error}", config_path.display()))?;
    let mut document = contents
        .parse::<DocumentMut>()
        .map_err(|error| format!("config is not valid TOML: {error}"))?;
    set_path(&mut document, &format!("paths.{key}"), toml_value(value));
    let candidate = document.to_string();
    let config: AppConfig = toml::from_str(&candidate)
        .map_err(|error| format!("updated config is invalid: {error}"))?;
    if let Some(diagnostic) = crate::app::commands::validate::validate_app_config(&config).first() {
        return Err(diagnostic.clone());
    }
    atomic_write(config_path, candidate.as_bytes())
}

fn operational_values(config: &AppConfig) -> Result<serde_json::Value, String> {
    let mut root = serde_json::Map::new();
    insert_serialized(&mut root, "runtime", &config.runtime)?;
    insert_serialized(&mut root, "subscriptions", &config.subscriptions)?;
    insert_serialized(&mut root, "routing", &config.routing)?;
    insert_serialized(&mut root, "dns", &config.dns)?;
    insert_serialized(&mut root, "testing", &config.testing)?;
    insert_serialized(&mut root, "server", &config.server)?;
    insert_serialized(&mut root, "parser", &config.parser)?;
    Ok(serde_json::Value::Object(root))
}

fn insert_serialized<T: Serialize>(
    root: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: &T,
) -> Result<(), String> {
    root.insert(
        key.to_string(),
        serde_json::to_value(value)
            .map_err(|error| format!("could not prepare settings: {error}"))?,
    );
    Ok(())
}

fn flatten_settings(
    prefix: &str,
    current: &serde_json::Value,
    defaults: &serde_json::Value,
    document: &DocumentMut,
    output: &mut Vec<EditableSetting>,
) {
    let serde_json::Value::Object(entries) = current else {
        return;
    };
    for (key, current_value) in entries {
        let path = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };
        let default_value = defaults.get(key).unwrap_or(&serde_json::Value::Null);
        if SECRET_PATHS.contains(&path.as_str()) {
            output.push(build_setting(
                path,
                json_secret_value(current_value),
                json_secret_value(default_value),
                SettingKind::Secret,
                document,
            ));
        } else if current_value.is_object() {
            flatten_settings(&path, current_value, default_value, document, output);
        } else if let Some(value) = json_setting_value(&path, current_value) {
            let default = json_setting_value(&path, default_value).unwrap_or_else(|| value.clone());
            let kind = setting_kind(&path, &value);
            output.push(build_setting(path, value, default, kind, document));
        }
    }
}

fn build_setting(
    path: String,
    value: SettingValue,
    default_value: SettingValue,
    kind: SettingKind,
    document: &DocumentMut,
) -> EditableSetting {
    let section = path
        .rsplit_once('.')
        .map(|(section, _)| section)
        .unwrap_or("general")
        .to_string();
    let label = setting_label(&path);
    EditableSetting {
        effect: setting_effect(&path),
        help: help::for_path(&path).unwrap_or(help::FALLBACK),
        explicit: path_exists(document, &path),
        path,
        section,
        label,
        kind,
        value: value.clone(),
        default_value,
        original_value: value,
        reset: false,
    }
}

fn setting_label(path: &str) -> String {
    let key = match path {
        "parser.parse_mode" => "mode",
        "runtime.fragment.packets_mode" => "packet_mode",
        "runtime.log.dns_log" => "dns_logging",
        "runtime.mux.xudp_proxy_udp443" => "udp_443_handling",
        "runtime.rotation.health_failure_threshold" => "failure_threshold",
        _ => path.rsplit('.').next().unwrap_or(path),
    };
    let key = [
        "_milliseconds",
        "_seconds",
        "_minutes",
        "_hours",
        "_ms",
        "_secs",
    ]
    .into_iter()
    .find_map(|suffix| key.strip_suffix(suffix))
    .unwrap_or(key);

    key.split('_')
        .map(|word| match word {
            "api" | "asn" | "dns" | "http" | "https" | "icmp" | "ip" | "pac" | "tcp" | "tls"
            | "ttl" | "udp" | "url" | "xudp" => word.to_ascii_uppercase(),
            "db" => "Database".to_string(),
            "dest" => "Destination".to_string(),
            "dir" => "Directory".to_string(),
            "geoip" => "GeoIP".to_string(),
            _ => word.to_string(),
        })
        .enumerate()
        .map(|(index, word)| {
            if index != 0 || word.chars().all(|character| character.is_uppercase()) {
                return word;
            }
            let mut characters = word.chars();
            characters
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + characters.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn json_setting_value(path: &str, value: &serde_json::Value) -> Option<SettingValue> {
    match value {
        serde_json::Value::Bool(value) => Some(SettingValue::Bool(*value)),
        serde_json::Value::Number(value) => value.as_i64().map(SettingValue::Integer),
        serde_json::Value::String(value) => Some(SettingValue::Text(value.clone())),
        serde_json::Value::Array(values) => Some(SettingValue::List(
            values
                .iter()
                .filter_map(|value| match value {
                    serde_json::Value::String(value) => Some(value.clone()),
                    serde_json::Value::Number(value) => Some(value.to_string()),
                    _ => None,
                })
                .collect(),
        )),
        serde_json::Value::Null if OPTIONAL_LIST_PATHS.contains(&path) => {
            Some(SettingValue::List(Vec::new()))
        }
        _ => None,
    }
}

fn json_secret_value(value: &serde_json::Value) -> SettingValue {
    let value = match value {
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Object(value) => value
            .get("env")
            .and_then(serde_json::Value::as_str)
            .map(|name| format!("env:{name}"))
            .unwrap_or_default(),
        _ => String::new(),
    };
    SettingValue::Secret(value)
}

fn setting_kind(path: &str, value: &SettingValue) -> SettingKind {
    if let Some(options) = enum_options(path) {
        return SettingKind::Enum(options);
    }
    match value {
        SettingValue::Bool(_) => SettingKind::Bool,
        SettingValue::Integer(_) => SettingKind::Integer,
        SettingValue::Text(_) => SettingKind::Text,
        SettingValue::List(_) => SettingKind::List {
            numeric: NUMERIC_LIST_PATHS.contains(&path),
        },
        SettingValue::Secret(_) => SettingKind::Secret,
    }
}

fn enum_options(path: &str) -> Option<&'static [&'static str]> {
    match path {
        "runtime.engine" => Some(&["xray", "v2ray", "sing-box"]),
        "runtime.log.level" => Some(&["debug", "info", "warning", "error", "none"]),
        "runtime.shadowsocks.method" => Some(&[
            "aes-128-gcm",
            "aes-256-gcm",
            "chacha20-poly1305",
            "2022-blake3-aes-128-gcm",
            "2022-blake3-aes-256-gcm",
        ]),
        "runtime.shadowsocks.network" => Some(&["tcp", "udp", "tcp,udp"]),
        "runtime.mux.xudp_proxy_udp443" => Some(&["reject", "allow", "skip"]),
        "runtime.fragment.packets_mode" => Some(&["tlshello", "range"]),
        "routing.domain_strategy" => Some(&["AsIs", "IPIfNonMatch", "IPOnDemand"]),
        "dns.query_strategy" => Some(&["UseIP", "UseIPv4", "UseIPv6", "UseSystem"]),
        "testing.failure_policy" => Some(&["continue", "skip_remaining", "mark_failed"]),
        "testing.geoip.backend" | "testing.geoip.fallback" => {
            Some(&["mmdb", "ip-whois", "ip-api", "chain", "none"])
        }
        "testing.geoip.remote.provider" => Some(&["ip-whois", "ip-api"]),
        "parser.parse_mode" => Some(&["strict", "lenient", "auto", "loose"]),
        _ => None,
    }
}

fn setting_effect(path: &str) -> SettingEffect {
    if path.starts_with("runtime.rotation.")
        || path.starts_with("subscriptions.")
        || path.starts_with("server.")
    {
        SettingEffect::DaemonRestart
    } else if (path.starts_with("runtime.") && path != "runtime.replace_active_session")
        || path.starts_with("routing.")
        || path.starts_with("dns.")
    {
        SettingEffect::RuntimeRestart
    } else {
        SettingEffect::Live
    }
}

fn setting_item(setting: &EditableSetting) -> Result<Item, String> {
    match &setting.value {
        SettingValue::Bool(value) => Ok(toml_value(*value)),
        SettingValue::Integer(number) => Ok(toml_value(*number)),
        SettingValue::Text(text) => Ok(toml_value(text.clone())),
        SettingValue::List(values) => {
            let mut array = Array::new();
            let numeric = matches!(setting.kind, SettingKind::List { numeric: true });
            for entry in values {
                if numeric {
                    array.push(
                        entry
                            .parse::<i64>()
                            .map_err(|_| format!("{} must contain whole numbers", setting.path))?,
                    );
                } else {
                    array.push(entry.as_str());
                }
            }
            Ok(Item::Value(Value::Array(array)))
        }
        SettingValue::Secret(secret) => {
            if let Some(name) = secret.strip_prefix("env:") {
                let name = name.trim();
                if name.is_empty() {
                    return Err(format!(
                        "{} needs an environment variable name",
                        setting.path
                    ));
                }
                let mut table = InlineTable::new();
                table.insert("env", Value::from(name));
                Ok(Item::Value(Value::InlineTable(table)))
            } else {
                Ok(toml_value(secret.clone()))
            }
        }
    }
}

fn set_path(document: &mut DocumentMut, path: &str, item: Item) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut table = document.as_table_mut();
    for part in &parts[..parts.len() - 1] {
        if !table.contains_key(part) || !table[*part].is_table() {
            table.insert(part, Item::Table(Table::new()));
        }
        table = table[*part]
            .as_table_mut()
            .expect("inserted table should be a table");
    }
    table.insert(parts[parts.len() - 1], item);
}

fn remove_path(document: &mut DocumentMut, path: &str) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut table = document.as_table_mut();
    for part in &parts[..parts.len() - 1] {
        let Some(next) = table.get_mut(part).and_then(Item::as_table_mut) else {
            return;
        };
        table = next;
    }
    table.remove(parts[parts.len() - 1]);
}

fn path_exists(document: &DocumentMut, path: &str) -> bool {
    let mut item: &Item = document.as_item();
    for part in path.split('.') {
        let Some(next) = item.get(part) else {
            return false;
        };
        item = next;
    }
    !item.is_none()
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let permissions = fs::metadata(path)
        .ok()
        .map(|metadata| metadata.permissions());
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .map_err(|error| format!("could not create temporary config: {error}"))?;
    temporary
        .write_all(contents)
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|error| format!("could not write temporary config: {error}"))?;
    if let Some(permissions) = permissions {
        temporary
            .as_file()
            .set_permissions(permissions)
            .map_err(|error| format!("could not preserve config permissions: {error}"))?;
    }
    temporary
        .persist(path)
        .map_err(|error| format!("could not replace config: {}", error.error))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(contents: &str) -> (tempfile::TempDir, ConfigEditSession) {
        let root = tempfile::tempdir().expect("temp directory should be created");
        let path = root.path().join("config.toml");
        fs::write(&path, contents).expect("config should be written");
        let session = ConfigEditSession::open(&path).expect("session should open");
        (root, session)
    }

    #[test]
    fn updates_runtime_binary_path_without_losing_comments() {
        let root = tempfile::tempdir().expect("temp directory should be created");
        let path = root.path().join("config.toml");
        fs::write(&path, "# keep me\n[runtime]\nengine = \"xray\"\n")
            .expect("config should be written");

        update_runtime_binary_path(&path, "xray", Path::new("/tmp/managed/xray"))
            .expect("path should update");

        let contents = fs::read_to_string(path).expect("config should be readable");
        assert!(contents.contains("# keep me"));
        assert!(contents.contains("xray = \"/tmp/managed/xray\""));
    }

    #[test]
    fn exposes_supported_roots_and_effective_defaults() {
        let (_root, session) = session("# keep me\n[database]\nbackend = \"sqlite\"\n");
        let editable_roots = [
            "runtime",
            "subscriptions",
            "routing",
            "dns",
            "testing",
            "server",
            "parser",
        ];

        assert!(session.settings.iter().all(|setting| {
            editable_roots
                .iter()
                .any(|root| setting.path.starts_with(root))
        }));
        assert!(
            session
                .settings
                .iter()
                .any(|setting| setting.path == "runtime.socks.port"
                    && setting.value == SettingValue::Integer(18200))
        );
        assert!(
            !session
                .settings
                .iter()
                .any(|setting| setting.path.starts_with("database."))
        );
        assert!(
            !session
                .settings
                .iter()
                .any(|setting| setting.path.starts_with("dns.hosts."))
        );
    }

    #[test]
    fn help_metadata_covers_every_exposed_setting() {
        let (_root, session) = session("");
        let missing: Vec<&str> = session
            .settings
            .iter()
            .filter(|setting| help::for_path(&setting.path).is_none())
            .map(|setting| setting.path.as_str())
            .collect();
        assert!(missing.is_empty(), "missing setting help for {missing:?}");

        for setting in &session.settings {
            assert!(
                !setting.help.description.trim().is_empty(),
                "{}",
                setting.path
            );
            assert!(setting.help.example.contains(" = "), "{}", setting.path);
            assert!(
                !setting.possible_values().trim().is_empty(),
                "{}",
                setting.path
            );
            if let SettingKind::Enum(options) = &setting.kind {
                let possible_values = setting.possible_values();
                assert!(
                    options
                        .iter()
                        .all(|option| possible_values.contains(option)),
                    "{}",
                    setting.path
                );
            }
        }
    }

    #[test]
    fn settings_use_concise_humanized_labels() {
        let (_root, session) = session("");
        let label = |path: &str| {
            session
                .settings
                .iter()
                .find(|setting| setting.path == path)
                .unwrap_or_else(|| panic!("missing setting {path}"))
                .label
                .as_str()
        };

        assert_eq!(label("parser.parse_mode"), "Mode");
        assert_eq!(label("dns.query_strategy"), "Query strategy");
        assert_eq!(label("runtime.fragment.packets_mode"), "Packet mode");
        assert_eq!(label("runtime.log.dns_log"), "DNS logging");
        assert_eq!(label("runtime.mux.xudp_proxy_udp443"), "UDP 443 handling");
        assert_eq!(
            label("runtime.rotation.health_failure_threshold"),
            "Failure threshold"
        );
        assert_eq!(label("server.pac_allowed_hosts"), "PAC allowed hosts");
        assert_eq!(label("testing.geoip.remote.timeout_ms"), "Timeout");
        assert_eq!(label("testing.geoip.cache.ttl_secs"), "TTL");
    }

    #[test]
    fn settings_expose_origin_and_default_metadata() {
        let (_root, session) = session("[runtime.socks]\nport = 1080\n");
        let port = session
            .settings
            .iter()
            .find(|setting| setting.path == "runtime.socks.port")
            .expect("port setting");
        let dns = session
            .settings
            .iter()
            .find(|setting| setting.path == "dns.servers")
            .expect("DNS setting");

        assert!(port.is_explicit());
        assert_eq!(port.default_value(), &SettingValue::Integer(18200));
        assert!(!dns.is_explicit());
        assert_eq!(dns.default_value(), &SettingValue::List(Vec::new()));
    }

    #[test]
    fn saves_routing_and_dns_without_touching_dynamic_hosts() {
        let contents = "[dns]\nquery_strategy = \"UseSystem\"\n\
                        [dns.hosts]\n\"domain:example.test\" = \"127.0.0.1\"\n\
                        [routing]\ndomain_strategy = \"IPIfNonMatch\"\n";
        let (root, mut session) = session(contents);
        session
            .settings
            .iter_mut()
            .find(|setting| setting.path == "dns.query_strategy")
            .expect("DNS strategy setting")
            .cycle_enum(1);
        session
            .settings
            .iter_mut()
            .find(|setting| setting.path == "routing.direct.domain")
            .expect("direct-domain setting")
            .set_from_input("example.com, domain:internal")
            .expect("domain list should parse");

        let outcome = session.save().expect("save should succeed");
        let saved =
            fs::read_to_string(root.path().join("config.toml")).expect("config should read");
        assert!(saved.contains("\"domain:example.test\" = \"127.0.0.1\""));
        assert!(saved.contains("query_strategy = \"UseIP\""));
        assert!(saved.contains("domain = [\"example.com\", \"domain:internal\"]"));
        assert_eq!(outcome.config.dns.query_strategy, "UseIP");
        assert_eq!(
            outcome.config.routing.direct.domain,
            ["example.com", "domain:internal"]
        );
        assert_eq!(
            outcome.effects,
            [SettingEffect::RuntimeRestart].into_iter().collect()
        );
    }

    #[test]
    fn saves_only_changed_key_and_preserves_comments() {
        let (root, mut session) = session("# keep me\n[runtime.socks]\nport = 18200\n");
        let setting = session
            .settings
            .iter_mut()
            .find(|setting| setting.path == "runtime.socks.port")
            .expect("port setting");
        setting.set_from_input("1080").expect("input should parse");

        let outcome = session.save().expect("save should succeed");
        let saved =
            fs::read_to_string(root.path().join("config.toml")).expect("config should read");
        assert!(saved.contains("# keep me"));
        assert!(saved.contains("port = 1080"));
        assert_eq!(outcome.config.runtime.socks.port, 1080);
        assert_eq!(outcome.changed_paths, ["runtime.socks.port"]);
    }

    #[cfg(unix)]
    #[test]
    fn save_without_changes_does_not_replace_config_file() {
        use std::os::unix::fs::MetadataExt;

        let (root, mut session) = session("[runtime.socks]\nport = 1080\n");
        let path = root.path().join("config.toml");
        let inode = fs::metadata(&path).expect("metadata").ino();

        let outcome = session.save().expect("unchanged save should succeed");

        assert!(outcome.changed_paths.is_empty());
        assert_eq!(fs::metadata(path).expect("metadata").ino(), inode);
    }

    #[test]
    fn reset_removes_explicit_override() {
        let (root, mut session) = session("[runtime.socks]\nport = 1080\n");
        session
            .settings
            .iter_mut()
            .find(|setting| setting.path == "runtime.socks.port")
            .expect("port setting")
            .reset_to_default();

        let outcome = session.save().expect("save should succeed");
        let saved =
            fs::read_to_string(root.path().join("config.toml")).expect("config should read");
        assert!(!saved.contains("port ="));
        assert_eq!(outcome.config.runtime.socks.port, 18200);
    }

    #[test]
    fn rejects_external_changes_without_overwriting_them() {
        let (root, mut session) = session("[runtime]\nengine = \"xray\"\n");
        session
            .settings
            .iter_mut()
            .find(|setting| setting.path == "runtime.engine")
            .expect("engine setting")
            .cycle_enum(1);
        let path = root.path().join("config.toml");
        fs::write(&path, "# external\n[runtime]\nengine = \"xray\"\n")
            .expect("external edit should write");

        let error = session.save().expect_err("save should reject conflict");
        assert!(error.contains("changed on disk"));
        assert!(
            fs::read_to_string(path)
                .expect("config should read")
                .contains("# external")
        );
    }

    #[test]
    fn semantic_validation_prevents_invalid_cross_field_save() {
        let (root, mut session) = session("# original\n");
        session
            .settings
            .iter_mut()
            .find(|setting| setting.path == "runtime.http.enabled")
            .expect("http enabled setting")
            .toggle();
        session
            .settings
            .iter_mut()
            .find(|setting| setting.path == "runtime.http.port")
            .expect("http port setting")
            .set_from_input("18200")
            .expect("port should parse");

        let error = session.save().expect_err("duplicate ports should fail");
        assert!(error.contains("[runtime.http].port"));
        assert_eq!(
            fs::read_to_string(root.path().join("config.toml")).expect("config should read"),
            "# original\n"
        );
    }

    #[test]
    fn saves_environment_backed_secret_without_exposing_a_literal() {
        let (root, mut session) = session("[server]\nenabled = false\n");
        session
            .settings
            .iter_mut()
            .find(|setting| setting.path == "server.key")
            .expect("server key setting")
            .set_from_input("env:XRAT_SERVER_KEY")
            .expect("secret should parse");

        let outcome = session.save().expect("save should succeed");
        let saved =
            fs::read_to_string(root.path().join("config.toml")).expect("config should read");
        assert!(saved.contains("key = { env = \"XRAT_SERVER_KEY\" }"));
        assert!(outcome.effects.contains(&SettingEffect::DaemonRestart));
    }
}
