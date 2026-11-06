use crate::app::{Result, context::AppContext, import};
use std::collections::BTreeMap;
use xrat_db::{ImportSource, ImportSummary};
use xrat_model::Node;

pub(crate) async fn persist_nodes(
    context: &AppContext,
    source: ImportSource,
    nodes: &[Node],
    name: Option<&str>,
    headers: Option<&BTreeMap<String, String>>,
) -> Result<ImportSummary> {
    if nodes.is_empty() {
        return Err(crate::app::AppError::NoSupportedConfig);
    }
    #[cfg(any(windows, test))]
    if let Some(client) = &context.machine_client {
        use super::machine_service::{CatalogPayload, CatalogRequest};
        return match client
            .catalog(CatalogRequest::Import {
                contents: vec![
                    nodes
                        .iter()
                        .map(|node| node.raw_config.as_str())
                        .collect::<Vec<_>>()
                        .join("\n"),
                ],
                source,
                name: name.map(str::to_owned),
                headers: headers.cloned().unwrap_or_default(),
            })
            .await?
        {
            CatalogPayload::Import(summary) => Ok(summary),
            _ => Err(crate::app::AppError::InvalidArgument(
                "unexpected machine import response".into(),
            )),
        };
    }
    import::persist_nodes(&context.db, source, nodes, name, headers).await
}

pub(crate) async fn add(context: &AppContext, input: &str) -> Result<ImportSummary> {
    let (source, node) = import::load_single_node(input)?;
    persist_nodes(context, source, &[node], None, None).await
}

pub(crate) async fn subscription(
    context: &AppContext,
    input: &str,
    name: Option<&str>,
    headers: &BTreeMap<String, String>,
) -> Result<ImportSummary> {
    subscription_with_fallback(context, input, name, None, headers).await
}

pub(crate) async fn subscription_with_fallback(
    context: &AppContext,
    input: &str,
    name: Option<&str>,
    fallback_name: Option<&str>,
    headers: &BTreeMap<String, String>,
) -> Result<ImportSummary> {
    #[cfg(any(windows, test))]
    if let Some(client) = &context.machine_client {
        return client
            .import_subscription_with_fallback(input, name, fallback_name, headers)
            .await;
    }
    let (mut source, bytes) = import::load_subscription_input(input, headers).await?;
    source.name = source.name.or_else(|| fallback_name.map(str::to_owned));
    persist_content(context, source, &bytes, name, headers).await
}

pub(crate) async fn persist_content(
    context: &AppContext,
    source: ImportSource,
    bytes: &[u8],
    name: Option<&str>,
    headers: &BTreeMap<String, String>,
) -> Result<ImportSummary> {
    if let Some(document) = super::xray_profiles::parse(bytes)? {
        return super::xray_profiles::import(context, &source, &document, name).await;
    }
    let nodes = import::parse_nodes(bytes)?;
    let stored_headers = matches!(source.kind, xrat_db::SourceKind::Url).then_some(headers);
    persist_nodes(context, source, &nodes, name, stored_headers).await
}
