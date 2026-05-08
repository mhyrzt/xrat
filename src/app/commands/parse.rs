use std::fs;
use std::io::Read;

use crate::app::AppError;
use crate::cli::{ParseArgs, ParseEngine};
use crate::config::parse_link;
use crate::model::Node;
use crate::singbox::generate_parse_config as generate_singbox_parse_config;
use crate::xray::generate_runtime_config_for_inbounds;

pub async fn run(args: &ParseArgs) -> crate::app::Result<()> {
    validate_inputs(args)?;
    let links = load_links(args)?;

    if links.is_empty() {
        return Err(AppError::NoSupportedConfig);
    }

    if args.json && links.len() != 1 {
        return Err(AppError::InvalidArgument(
            "--json supports exactly one config link".to_string(),
        ));
    }

    for (index, link) in links.iter().enumerate() {
        let scheme = parse_scheme(link)?;
        let node = parse_link(link).map_err(|error| {
            AppError::InvalidArgument(format!("parse failed for link #{}: {error}", index + 1))
        })?;
        let node = node.ok_or(AppError::NoSupportedConfig)?;
        let engine = resolve_engine(args.engine, &scheme)?;

        if args.json {
            print_json(&node, engine)?;
        } else {
            if links.len() > 1 {
                println!("Config #{}", index + 1);
            }
            print_details(&node, engine);
        }
    }

    Ok(())
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

fn load_links(args: &ParseArgs) -> crate::app::Result<Vec<String>> {
    if let Some(input) = &args.input {
        return Ok(vec![input.trim().to_string()]);
    }

    if let Some(path) = &args.file {
        let body = fs::read_to_string(path)?;
        return Ok(extract_links(&body));
    }

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    Ok(extract_links(&input))
}

fn extract_links(input: &str) -> Vec<String> {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToString::to_string)
        .collect()
}

fn parse_scheme(link: &str) -> crate::app::Result<String> {
    let parsed = url::Url::parse(link)
        .map_err(|error| AppError::InvalidArgument(format!("invalid config URL: {error}")))?;
    Ok(parsed.scheme().to_string())
}

fn resolve_engine(requested: ParseEngine, scheme: &str) -> crate::app::Result<ParseEngine> {
    match requested {
        ParseEngine::Auto => {
            if matches!(scheme, "hysteria2" | "hy2") {
                Ok(ParseEngine::SingBox)
            } else {
                Ok(ParseEngine::Xray)
            }
        }
        ParseEngine::Xray => {
            if matches!(scheme, "hysteria2" | "hy2") {
                return Err(AppError::InvalidArgument(
                    "hysteria2/hy2 is not compatible with xray engine".to_string(),
                ));
            }
            Ok(ParseEngine::Xray)
        }
        ParseEngine::SingBox => Ok(ParseEngine::SingBox),
    }
}

fn print_json(node: &Node, engine: ParseEngine) -> crate::app::Result<()> {
    let value = build_json_value(node, engine)?;
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn build_json_value(node: &Node, engine: ParseEngine) -> crate::app::Result<serde_json::Value> {
    match engine {
        ParseEngine::Xray => {
            let config =
                generate_runtime_config_for_inbounds(node, Some(("127.0.0.1", 1080, false)), None)
                    .map_err(AppError::InvalidArgument)?;
            Ok(serde_json::to_value(config)?)
        }
        ParseEngine::SingBox => {
            let config =
                generate_singbox_parse_config(node, 1080).map_err(AppError::InvalidArgument)?;
            Ok(serde_json::to_value(config)?)
        }
        ParseEngine::Auto => unreachable!("auto is resolved before output"),
    }
}

fn print_details(node: &Node, engine: ParseEngine) {
    println!("  engine: {engine}");
    println!("protocol: {}", node.protocol);
    println!(" address: {}", node.address);
    println!("    port: {}", node.port);
    println!(" network: {}", node.network);
    println!("     tls: {}", node.tls.as_deref().unwrap_or("none"));
    println!("     sni: {}", node.sni.as_deref().unwrap_or("-"));
    println!("    host: {}", node.host.as_deref().unwrap_or("-"));
    println!("    path: {}", node.path.as_deref().unwrap_or("-"));
    println!("    name: {}", node.name.as_deref().unwrap_or("-"));
}

#[cfg(test)]
mod tests {
    use super::{build_json_value, resolve_engine};
    use crate::cli::ParseEngine;
    use crate::config::parse_link;

    #[test]
    fn resolves_auto_engine_for_hy2() {
        let engine = resolve_engine(ParseEngine::Auto, "hy2").expect("engine should resolve");
        assert!(matches!(engine, ParseEngine::SingBox));
    }

    #[test]
    fn resolves_auto_engine_for_vless() {
        let engine = resolve_engine(ParseEngine::Auto, "vless").expect("engine should resolve");
        assert!(matches!(engine, ParseEngine::Xray));
    }

    #[test]
    fn rejects_hy2_with_xray_engine() {
        let error =
            resolve_engine(ParseEngine::Xray, "hysteria2").expect_err("hy2 must reject xray");
        assert!(
            error
                .to_string()
                .contains("not compatible with xray engine")
        );
    }

    #[test]
    fn builds_xray_json_for_vless() {
        let link = "vless://uuid-123@example.com:443?type=ws&security=tls&sni=cdn.example.com&path=%2Fray#Node";
        let node = parse_link(link)
            .expect("link should parse")
            .expect("node should exist");
        let value = build_json_value(&node, ParseEngine::Xray).expect("json should build");
        assert_eq!(value["outbounds"][0]["protocol"], "vless");
        assert_eq!(value["inbounds"][0]["protocol"], "socks");
    }

    #[test]
    fn builds_singbox_json_for_hy2() {
        let link = "hy2://secret@example.com:443?sni=edge.example.com&obfs=salamander&obfs-password=test&insecure=1&alpn=h3,h2#HY2";
        let node = parse_link(link)
            .expect("link should parse")
            .expect("node should exist");
        let value = build_json_value(&node, ParseEngine::SingBox).expect("json should build");
        assert_eq!(value["outbounds"][0]["type"], "hysteria2");
        assert_eq!(value["outbounds"][0]["tls"]["insecure"], true);
    }
}
