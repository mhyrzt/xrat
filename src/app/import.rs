use crate::app::AppError;
use crate::app::input::source::read_input;
use crate::config::parse_text;
use crate::db::ImportSource;
use crate::db::SourceKind;
use crate::model::Node;
use crate::support::decode::decode_or_raw_text;

pub fn load_nodes(input: &str) -> crate::app::Result<(ImportSource, Vec<Node>)> {
    let (source, input_data) = read_input(input)?;
    let config_text = decode_or_raw_text(&input_data)?;
    reject_raw_json_config(&config_text)?;
    let normalized_text = expand_url_list(&config_text)?;

    Ok((source, parse_text(&normalized_text)))
}

pub fn load_single_node(input: &str) -> crate::app::Result<(ImportSource, Node)> {
    reject_raw_json_config(input)?;

    let mut nodes = parse_text(input).into_iter();
    let Some(node) = nodes.next() else {
        return Err(AppError::NoSupportedConfig);
    };

    if nodes.next().is_some() {
        return Err(AppError::MultipleConfigsForAdd);
    }

    Ok((
        ImportSource {
            kind: SourceKind::RawText,
            value: input.to_string(),
            name: node.name.clone(),
        },
        node,
    ))
}

fn reject_raw_json_config(config_text: &str) -> crate::app::Result<()> {
    if serde_json::from_str::<serde_json::Value>(config_text).is_ok() {
        return Err(AppError::RawJsonImportUnsupported);
    }

    Ok(())
}

fn expand_url_list(input: &str) -> crate::app::Result<String> {
    let mut collected = Vec::new();
    let mut saw_url = false;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if looks_like_url(trimmed) {
            saw_url = true;
            let (_, body) = read_input(trimmed)?;
            collected.push(decode_or_raw_text(&body)?);
        } else {
            collected.push(trimmed.to_string());
        }
    }

    if saw_url {
        Ok(collected.join("\n"))
    } else {
        Ok(input.to_string())
    }
}

fn looks_like_url(input: &str) -> bool {
    matches!(
        input.split_once("://").map(|(scheme, _)| scheme),
        Some("http" | "https")
    )
}

#[cfg(test)]
mod tests {
    use super::load_single_node;
    use crate::db::SourceKind;
    use crate::model::Protocol;

    #[test]
    fn parses_single_manual_config_as_raw_text_source() {
        let (source, node) = load_single_node(
            "vless://uuid-123@example.com:443?type=ws&security=tls#Example%20Node",
        )
        .expect("single config should parse");

        assert_eq!(source.kind, SourceKind::RawText);
        assert_eq!(node.protocol, Protocol::Vless);
        assert_eq!(node.address, "example.com");
    }

    #[test]
    fn rejects_multiple_configs_for_add() {
        let err = load_single_node("vless://uuid-123@example.com:443#One\nss://YWVzLTI1Ni1nY206c2VjcmV0@example.com:8388#Two")
            .expect_err("multiple configs should fail");

        assert!(err.to_string().contains("exactly one"));
    }
}
