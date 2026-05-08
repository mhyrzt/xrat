use std::fs;
use std::io::Read;

use crate::app::AppError;
use crate::cli::{ParseArgs, ParseEngine};
use crate::config::{EngineMode, ParsedEntry, ResolvedEngine, parse_links_batch};
use crate::model::Node;
use crate::singbox::generate_parse_config as generate_singbox_parse_config;
use crate::support::decode::decode_or_raw_text;
use crate::xray::generate_runtime_config_for_inbounds;

pub async fn run(args: &ParseArgs) -> crate::app::Result<()> {
    validate_inputs(args)?;
    let inputs = load_inputs(args)?;

    if inputs.is_empty() {
        return Err(AppError::NoSupportedConfig);
    }

    if args.json && inputs.len() != 1 {
        return Err(AppError::InvalidArgument(
            "--json supports exactly one config link".to_string(),
        ));
    }

    let parsed = parse_inputs(&inputs, args.engine)?;

    for (index, entry) in parsed.iter().enumerate() {
        if args.json {
            print_json(&entry.node, entry.engine)?;
        } else {
            if inputs.len() > 1 {
                println!("Config #{}", index + 1);
            }
            print_details(&entry.node, entry.engine);
        }
    }

    Ok(())
}

fn parse_inputs(
    inputs: &[ParseInput],
    requested_engine: ParseEngine,
) -> crate::app::Result<Vec<ParsedEntry>> {
    let batch_inputs: Vec<(String, String)> = inputs
        .iter()
        .map(|input| (input.source.clone(), input.link.clone()))
        .collect();
    let parsed = parse_links_batch(&batch_inputs, to_engine_mode(requested_engine))
        .map_err(|error| AppError::InvalidArgument(error.to_string()))?;

    if parsed.is_empty() {
        return Err(AppError::NoSupportedConfig);
    }

    Ok(parsed)
}

fn validate_inputs(args: &ParseArgs) -> crate::app::Result<()> {
    let mut modes = 0u8;
    if args.input.is_some() {
        modes += 1;
    }
    if args.file.is_some() {
        modes += 1;
    }
    if args.stdin {
        modes += 1;
    }

    if modes == 0 {
        return Err(AppError::InvalidArgument(
            "provide one input: positional <input>, --file, or --stdin".to_string(),
        ));
    }
    if modes > 1 {
        return Err(AppError::InvalidArgument(
            "use only one input mode: positional <input>, --file, or --stdin".to_string(),
        ));
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct ParseInput {
    link: String,
    source: String,
}

fn load_inputs(args: &ParseArgs) -> crate::app::Result<Vec<ParseInput>> {
    if let Some(input) = &args.input {
        return Ok(vec![ParseInput {
            link: input.trim().to_string(),
            source: "arg".to_string(),
        }]);
    }

    if let Some(path) = &args.file {
        let raw_bytes = fs::read(path)?;
        let decoded = decode_input_text(&raw_bytes, &format!("file {}", path.to_string_lossy()))?;
        return Ok(extract_inputs(
            &decoded,
            &format!("file {}", path.to_string_lossy()),
        ));
    }

    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input)?;
    let decoded = decode_input_text(&input, "stdin")?;
    Ok(extract_inputs(&decoded, "stdin"))
}

fn decode_input_text(raw: &[u8], source: &str) -> crate::app::Result<String> {
    decode_or_raw_text(raw).map_err(|error| {
        AppError::InvalidArgument(format!("failed to decode {source} input: {error}"))
    })
}

fn extract_inputs(input: &str, source_prefix: &str) -> Vec<ParseInput> {
    input
        .lines()
        .enumerate()
        .map(|(index, line)| (index + 1, line.trim()))
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
        .map(|(line_no, link)| ParseInput {
            link: link.to_string(),
            source: format!("{source_prefix} line {line_no}"),
        })
        .collect()
}

fn to_engine_mode(engine: ParseEngine) -> EngineMode {
    match engine {
        ParseEngine::Auto => EngineMode::Auto,
        ParseEngine::Xray => EngineMode::Xray,
        ParseEngine::SingBox => EngineMode::SingBox,
    }
}

fn clean_json_value(value: serde_json::Value) -> Option<serde_json::Value> {
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

fn print_json(node: &Node, engine: ResolvedEngine) -> crate::app::Result<()> {
    let value = build_json_value(node, engine)?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn build_json_value(node: &Node, engine: ResolvedEngine) -> crate::app::Result<serde_json::Value> {
    let raw_value = match engine {
        ResolvedEngine::Xray => {
            let config =
                generate_runtime_config_for_inbounds(node, Some(("127.0.0.1", 1080, false)), None)
                    .map_err(AppError::InvalidArgument)?;
            serde_json::to_value(config)?
        }
        ResolvedEngine::SingBox => {
            let config =
                generate_singbox_parse_config(node, 1080).map_err(AppError::InvalidArgument)?;
            serde_json::to_value(config)?
        }
    };

    clean_json_value(raw_value).ok_or(AppError::InvalidArgument(
        "generated parse JSON was empty after cleanup".to_string(),
    ))
}

fn print_details(node: &Node, engine: ResolvedEngine) {
    println!("{}", format_details(node, engine));
}

fn format_details(node: &Node, engine: ResolvedEngine) -> String {
    format!(
        "  engine: {engine}\nprotocol: {}\n address: {}\n    port: {}\n network: {}\n     tls: {}\n     sni: {}\n    host: {}\n    path: {}\n    name: {}",
        node.protocol,
        node.address,
        node.port,
        node.network,
        node.tls.as_deref().unwrap_or("none"),
        node.sni.as_deref().unwrap_or("-"),
        node.host.as_deref().unwrap_or("-"),
        node.path.as_deref().unwrap_or("-"),
        node.name.as_deref().unwrap_or("-")
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_json_value, decode_input_text, extract_inputs, format_details, parse_inputs,
        validate_inputs,
    };
    use crate::cli::ParseArgs;
    use crate::cli::ParseEngine;
    use crate::config::{ResolvedEngine, parse_link};
    use std::path::PathBuf;

    #[test]
    fn validate_inputs_rejects_missing_mode() {
        let args = ParseArgs {
            input: None,
            file: None,
            stdin: false,
            json: false,
            engine: ParseEngine::Auto,
        };
        let error = validate_inputs(&args).expect_err("missing mode should fail");
        assert!(error.to_string().contains("provide one input"));
    }

    #[test]
    fn validate_inputs_rejects_conflicting_modes() {
        let args = ParseArgs {
            input: Some("vless://example".to_string()),
            file: Some(PathBuf::from("/tmp/links.txt")),
            stdin: false,
            json: false,
            engine: ParseEngine::Auto,
        };
        let error = validate_inputs(&args).expect_err("conflict should fail");
        assert!(error.to_string().contains("use only one input mode"));
    }

    #[test]
    fn extract_inputs_tracks_line_context() {
        let parsed = extract_inputs("vless://one\n\n#comment\nvmess://two", "stdin");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].source, "stdin line 1");
        assert_eq!(parsed[1].source, "stdin line 4");
    }

    #[test]
    fn decode_input_accepts_base64_text_list() {
        let encoded = b"dmxlc3M6Ly9leGFtcGxlLmNvbTo0NDMKc3M6Ly9hYmNAZXhhbXBsZS5jb206ODM4OA==";
        let decoded = decode_input_text(encoded, "stdin").expect("base64 should decode");
        assert!(decoded.contains("vless://example.com:443"));
        assert!(decoded.contains("ss://abc@example.com:8388"));
    }

    #[test]
    fn cleans_null_and_empty_fields_from_json() {
        let link = "vless://uuid-123@example.com:443?type=ws&security=tls&sni=cdn.example.com&path=%2Fray#Node";
        let node = parse_link(link)
            .expect("link should parse")
            .expect("node should exist");
        let value = build_json_value(&node, ResolvedEngine::Xray).expect("json should build");
        assert!(
            !has_null_or_empty(&value),
            "expected cleaned JSON without null/empty fields"
        );
    }

    fn has_null_or_empty(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Null => true,
            serde_json::Value::String(text) => text.is_empty(),
            serde_json::Value::Object(map) => map.is_empty() || map.values().any(has_null_or_empty),
            serde_json::Value::Array(values) => {
                values.is_empty() || values.iter().any(has_null_or_empty)
            }
            _ => false,
        }
    }

    #[test]
    fn cleans_empty_array_and_object_values() {
        let value = serde_json::json!({
            "a": null,
            "b": "",
            "c": [],
            "d": {},
            "e": ["ok", null]
        });

        let cleaned = super::clean_json_value(value).expect("should keep non-empty values");
        assert_eq!(cleaned, serde_json::json!({"e": ["ok"]}));
    }

    #[test]
    fn parse_policy_stops_on_first_error() {
        let inputs = vec![
            super::ParseInput {
                link: "not-a-url".to_string(),
                source: "stdin line 1".to_string(),
            },
            super::ParseInput {
                link: "vless://uuid-123@example.com:443?type=tcp#ok".to_string(),
                source: "stdin line 2".to_string(),
            },
        ];

        let error = parse_inputs(&inputs, ParseEngine::Auto).expect_err("must stop on first bad");
        let rendered = error.to_string();
        assert!(rendered.contains("stdin line 1"));
        assert!(!rendered.contains("stdin line 2"));
    }

    #[test]
    fn formats_details_output_for_valid_link() {
        let link = "vless://uuid-123@example.com:443?type=ws&security=tls&sni=cdn.example.com#Node";
        let node = parse_link(link)
            .expect("link should parse")
            .expect("node should exist");

        let details = format_details(&node, ResolvedEngine::Xray);
        assert!(details.contains("engine: xray"));
        assert!(details.contains("protocol: vless"));
        assert!(details.contains("address: example.com"));
    }

    #[test]
    fn builds_xray_json_for_vless() {
        let link = "vless://uuid-123@example.com:443?type=ws&security=tls&sni=cdn.example.com&path=%2Fray#Node";
        let node = parse_link(link)
            .expect("link should parse")
            .expect("node should exist");
        let value = build_json_value(&node, ResolvedEngine::Xray).expect("json should build");
        assert_eq!(value["outbounds"][0]["protocol"], "vless");
        assert_eq!(value["inbounds"][0]["protocol"], "socks");
    }

    #[test]
    fn builds_singbox_json_for_hy2() {
        let link = "hy2://secret@example.com:443?sni=edge.example.com&obfs=salamander&obfs-password=test&insecure=1&alpn=h3,h2#HY2";
        let node = parse_link(link)
            .expect("link should parse")
            .expect("node should exist");
        let value = build_json_value(&node, ResolvedEngine::SingBox).expect("json should build");
        assert_eq!(value["outbounds"][0]["type"], "hysteria2");
        assert_eq!(value["outbounds"][0]["tls"]["insecure"], true);
    }
}
