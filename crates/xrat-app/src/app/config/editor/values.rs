use super::prelude::*;
use super::types::*;

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

pub(crate) fn operational_values(config: &AppConfig) -> Result<serde_json::Value, String> {
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

pub(crate) fn insert_serialized<T: Serialize>(
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

pub(crate) fn flatten_settings(
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
        if matches!(
            path.as_str(),
            "dns.resolvers" | "dns.rules" | "dns.outbound.rules"
        ) {
            continue;
        }
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

pub(crate) fn build_setting(
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
        help: super::help::for_path(&path).unwrap_or(super::help::FALLBACK),
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

pub(crate) fn setting_label(path: &str) -> String {
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

pub(crate) fn json_setting_value(path: &str, value: &serde_json::Value) -> Option<SettingValue> {
    match value {
        serde_json::Value::Bool(value) => Some(SettingValue::Bool(*value)),
        serde_json::Value::Number(value) => value.as_i64().map(SettingValue::Integer),
        serde_json::Value::String(value) => Some(SettingValue::Text(value.clone())),
        serde_json::Value::Array(values)
            if values
                .iter()
                .all(|value| value.is_string() || value.is_number()) =>
        {
            Some(SettingValue::List(
                values
                    .iter()
                    .filter_map(|value| match value {
                        serde_json::Value::String(value) => Some(value.clone()),
                        serde_json::Value::Number(value) => Some(value.to_string()),
                        _ => None,
                    })
                    .collect(),
            ))
        }
        serde_json::Value::Null if OPTIONAL_LIST_PATHS.contains(&path) => {
            Some(SettingValue::List(Vec::new()))
        }
        _ => None,
    }
}

pub(crate) fn json_secret_value(value: &serde_json::Value) -> SettingValue {
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

pub(crate) fn setting_kind(path: &str, value: &SettingValue) -> SettingKind {
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

pub(crate) fn enum_options(path: &str) -> Option<&'static [&'static str]> {
    match path {
        "runtime.engine" => Some(&["xray", "v2ray", "sing-box"]),
        "runtime.xray_compatibility" => Some(&["auto", "stable", "prerelease"]),
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
        "runtime.tun.stack" => Some(&["system", "gvisor", "mixed"]),
        "runtime.tun.split_mode" => Some(&["all", "blacklist", "whitelist"]),
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

pub(crate) fn setting_effect(path: &str) -> SettingEffect {
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

pub(crate) fn setting_item(setting: &EditableSetting) -> Result<Item, String> {
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

pub(crate) fn set_path(document: &mut DocumentMut, path: &str, item: Item) {
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

pub(crate) fn remove_path(document: &mut DocumentMut, path: &str) {
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

pub(crate) fn path_exists(document: &DocumentMut, path: &str) -> bool {
    let mut item: &Item = document.as_item();
    for part in path.split('.') {
        let Some(next) = item.get(part) else {
            return false;
        };
        item = next;
    }
    !item.is_none()
}

pub(crate) fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), String> {
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
