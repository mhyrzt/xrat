use crate::app::AppError;
use crate::app::input::source::{read_input, read_input_async};
use crate::config::parse_text;
use crate::db::ImportSource;
use crate::db::SourceKind;
use crate::db::{Database, ImportSummary};
use crate::model::Node;
use crate::support::decode::decode_or_raw_text;
use crate::support::url::looks_like_url;

pub fn load_nodes(input: &str) -> crate::app::Result<(ImportSource, Vec<Node>)> {
    let (source, input_data) = read_input(input)?;
    let config_text = decode_or_raw_text(&input_data)?;
    reject_raw_json_config(&config_text)?;
    let normalized_text = expand_url_list(&config_text)?;

    Ok((source, parse_text(&normalized_text)))
}

pub async fn load_nodes_async(input: &str) -> crate::app::Result<(ImportSource, Vec<Node>)> {
    let (source, input_data) = read_input_async(input).await?;
    let config_text = decode_or_raw_text(&input_data)?;
    reject_raw_json_config(&config_text)?;
    let normalized_text = expand_url_list_async(&config_text).await?;

    Ok((source, parse_text(&normalized_text)))
}

pub async fn persist_nodes(
    database: &Database,
    mut source: ImportSource,
    nodes: &[Node],
    name: Option<&str>,
) -> crate::app::Result<ImportSummary> {
    if let Some(name) = name {
        source.name = Some(name.to_string());
    }
    let summary = database.import_nodes(&source, nodes).await?;
    if let Some(name) = name {
        database
            .set_subscription_name(summary.subscription_id, name)
            .await?;
    }
    Ok(summary)
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

async fn expand_url_list_async(input: &str) -> crate::app::Result<String> {
    let mut collected = Vec::new();
    let mut saw_url = false;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if looks_like_url(trimmed) {
            saw_url = true;
            let (_, body) = read_input_async(trimmed).await?;
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

#[cfg(test)]
mod tests {
    use super::{load_single_node, persist_nodes};
    use crate::db::{Database, ImportSource, SourceKind};
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

    #[tokio::test]
    async fn supplied_name_is_persisted_and_updates_existing_url() {
        let root = tempfile::tempdir().expect("temp directory should be created");
        let database = Database::connect_sqlite(&root.path().join("db.sqlite"))
            .await
            .expect("database should connect");

        for name in ["First", "Second"] {
            let (_, node) = load_single_node("vless://uuid-123@example.com:443#One")
                .expect("config should parse");
            let source = ImportSource {
                kind: SourceKind::Url,
                value: "https://example.com/sub".to_string(),
                name: None,
            };
            persist_nodes(&database, source, &[node], Some(name))
                .await
                .expect("import should persist");
        }

        let subscriptions = database
            .list_subscriptions()
            .await
            .expect("subscriptions should load");
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0].name.as_deref(), Some("Second"));
    }

    #[tokio::test]
    async fn omitted_name_preserves_existing_import_behavior() {
        let root = tempfile::tempdir().expect("temp directory should be created");
        let database = Database::connect_sqlite(&root.path().join("db.sqlite"))
            .await
            .expect("database should connect");
        let (_, node) =
            load_single_node("vless://uuid-123@example.com:443#One").expect("config should parse");
        let source = ImportSource {
            kind: SourceKind::Url,
            value: "https://example.com/sub".to_string(),
            name: None,
        };

        persist_nodes(&database, source, &[node], None)
            .await
            .expect("import should persist");

        let subscription = database
            .list_subscriptions()
            .await
            .expect("subscriptions should load")
            .pop()
            .expect("subscription should exist");
        assert_eq!(subscription.name, None);
    }
}
