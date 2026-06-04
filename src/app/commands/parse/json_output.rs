use crate::app::AppError;
use crate::app::commands::output;
use crate::config::ResolvedEngine;
use crate::model::Node;
use crate::singbox::generate_singbox_probe_config;
use crate::xray::generate_runtime_config_for_inbounds;

pub(super) fn clean_json_value(value: serde_json::Value) -> Option<serde_json::Value> {
    match value {
        serde_json::Value::Object(map) => {
            let mut cleaned = serde_json::Map::new();
            for (key, value) in map {
                if let Some(value) = clean_json_value(value) {
                    cleaned.insert(key, value);
                }
            }
            if cleaned.is_empty() {
                None
            } else {
                Some(serde_json::Value::Object(cleaned))
            }
        }
        serde_json::Value::Array(values) => {
            let cleaned: Vec<_> = values.into_iter().filter_map(clean_json_value).collect();
            if cleaned.is_empty() {
                None
            } else {
                Some(serde_json::Value::Array(cleaned))
            }
        }
        serde_json::Value::String(value) if value.is_empty() => None,
        serde_json::Value::Null => None,
        other => Some(other),
    }
}

pub(super) fn print_json(node: &Node, engine: ResolvedEngine) -> crate::app::Result<()> {
    let value = build_json_value(node, engine)?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

pub(super) fn build_json_value(
    node: &Node,
    engine: ResolvedEngine,
) -> crate::app::Result<serde_json::Value> {
    let raw_value = match engine {
        ResolvedEngine::Xray => {
            let config =
                generate_runtime_config_for_inbounds(node, Some(("127.0.0.1", 1080, false)), None)
                    .map_err(AppError::InvalidArgument)?;
            serde_json::to_value(config)?
        }
        ResolvedEngine::SingBox => {
            let config =
                generate_singbox_probe_config(node, 1080).map_err(AppError::InvalidArgument)?;
            serde_json::to_value(config)?
        }
    };

    clean_json_value(raw_value).ok_or(AppError::InvalidArgument(
        "generated parse JSON was empty after cleanup".to_string(),
    ))
}

pub(super) fn print_details(node: &Node, engine: ResolvedEngine) {
    println!("{}", format_details(node, engine));
}

pub(super) fn format_details(node: &Node, engine: ResolvedEngine) -> String {
    output::format_kv(
        Some("Parsed config"),
        &[
            ("engine", engine.to_string()),
            ("protocol", node.protocol.to_string()),
            ("address", node.address.clone()),
            ("port", node.port.to_string()),
            ("network", node.network.clone()),
            ("tls", node.tls.as_deref().unwrap_or("none").to_string()),
            ("sni", output::dash(node.sni.as_deref())),
            ("host", output::dash(node.host.as_deref())),
            ("path", output::dash(node.path.as_deref())),
            ("name", output::dash(node.name.as_deref())),
        ],
        output::color_enabled(),
    )
}
