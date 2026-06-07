use crate::app::commands::output::{self, Align, Cell, Column, Style};
use crate::app::commands::resolve::resolve_subscription_id;
use crate::app::context::AppContext;
use crate::cli::{ListArgs, ListConfigsArgs, ListFormat, ListSubscriptionsArgs, ListTarget};
use crate::db::{ConfigListFilter, ConfigRecord, SubscriptionRecord};
use crate::support::refs::short_ref;
use std::collections::HashMap;

pub async fn run(context: &AppContext, command: &ListArgs) -> crate::app::Result<()> {
    match &command.target {
        ListTarget::Configs(filters) => print_configs(context, filters).await?,
        ListTarget::Subscriptions(filters) => print_subscriptions(context, filters).await?,
    }

    Ok(())
}

async fn print_configs(context: &AppContext, filters: &ListConfigsArgs) -> crate::app::Result<()> {
    let filter = build_config_list_filter(context, filters).await?;
    let configs = context.db.list_configs(&filter).await?;
    let subscriptions = context.db.list_subscriptions().await?;
    let subscription_refs = subscriptions
        .iter()
        .map(|subscription| (subscription.id, subscription.r#ref.as_str()))
        .collect::<HashMap<_, _>>();

    if configs.is_empty() {
        println!("{}", output::empty_message("No configs matched."));
        return Ok(());
    }

    println!(
        "{}",
        format_configs(&configs, &subscription_refs, filters.format)?
    );

    Ok(())
}

async fn print_subscriptions(
    context: &AppContext,
    filters: &ListSubscriptionsArgs,
) -> crate::app::Result<()> {
    let mut subscriptions = context.db.list_subscriptions().await?;
    if let Some(kind) = &filters.kind {
        subscriptions.retain(|subscription| subscription.source_kind == kind.as_str());
    }

    if subscriptions.is_empty() {
        println!("{}", output::empty_message("No subscriptions matched."));
        return Ok(());
    }

    println!("{}", format_subscriptions(&subscriptions, filters.format)?);

    Ok(())
}

fn format_configs(
    configs: &[ConfigRecord],
    subscription_refs: &HashMap<i64, &str>,
    format: ListFormat,
) -> crate::app::Result<String> {
    match format {
        ListFormat::Table => Ok(format_config_table(configs, subscription_refs)),
        ListFormat::Tsv => Ok(format_config_tsv(configs, subscription_refs)),
        ListFormat::Json => Ok(serde_json::to_string_pretty(
            &configs
                .iter()
                .map(|config| config_json(config, subscription_refs))
                .collect::<Vec<_>>(),
        )?),
    }
}

fn format_config_table(configs: &[ConfigRecord], subscription_refs: &HashMap<i64, &str>) -> String {
    let columns = [
        Column {
            header: "REF",
            align: Align::Left,
        },
        Column {
            header: "SUB",
            align: Align::Left,
        },
        Column {
            header: "STATUS",
            align: Align::Left,
        },
        Column {
            header: "PROTO",
            align: Align::Left,
        },
        Column {
            header: "ADDRESS",
            align: Align::Left,
        },
        Column {
            header: "PORT",
            align: Align::Right,
        },
        Column {
            header: "NAME",
            align: Align::Left,
        },
    ];
    let rows = configs
        .iter()
        .map(|config| {
            vec![
                Cell::plain(short_ref(&config.r#ref).to_string()),
                Cell::plain(subscription_ref_cell(
                    config.subscription_id,
                    subscription_refs,
                )),
                Cell::styled(
                    format_config_flags(config.is_enabled, config.is_active, config.is_deleted),
                    config_style(config),
                ),
                Cell::plain(config.protocol.clone()),
                Cell::plain(output::truncate(&output::dash(Some(&config.address)), 36)),
                Cell::plain(config.port.to_string()),
                Cell::plain(output::truncate(config.name.as_deref().unwrap_or("-"), 32)),
            ]
        })
        .collect::<Vec<_>>();

    output::format_table(&columns, &rows, output::color_enabled())
}

fn format_config_tsv(configs: &[ConfigRecord], subscription_refs: &HashMap<i64, &str>) -> String {
    let mut lines = Vec::with_capacity(configs.len() + 1);
    lines.push("ref\tsubscription_ref\tstatus\tprotocol\taddress\tport\tname".to_string());
    for config in configs {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            config.r#ref,
            subscription_ref_tsv_cell(config.subscription_id, subscription_refs),
            format_config_flags(config.is_enabled, config.is_active, config.is_deleted),
            config.protocol,
            config.address,
            config.port,
            tsv_cell(config.name.as_deref()),
        ));
    }
    lines.join("\n")
}

fn format_subscriptions(
    subscriptions: &[SubscriptionRecord],
    format: ListFormat,
) -> crate::app::Result<String> {
    match format {
        ListFormat::Table => Ok(format_subscription_table(subscriptions)),
        ListFormat::Tsv => Ok(format_subscription_tsv(subscriptions)),
        ListFormat::Json => Ok(serde_json::to_string_pretty(
            &subscriptions
                .iter()
                .map(subscription_json)
                .collect::<Vec<_>>(),
        )?),
    }
}

fn format_subscription_table(subscriptions: &[SubscriptionRecord]) -> String {
    let columns = [
        Column {
            header: "REF",
            align: Align::Left,
        },
        Column {
            header: "KIND",
            align: Align::Left,
        },
        Column {
            header: "CONFIGS",
            align: Align::Right,
        },
        Column {
            header: "NAME",
            align: Align::Left,
        },
        Column {
            header: "SOURCE",
            align: Align::Left,
        },
    ];
    let rows = subscriptions
        .iter()
        .map(|subscription| {
            vec![
                Cell::plain(short_ref(&subscription.r#ref).to_string()),
                Cell::plain(subscription.source_kind.clone()),
                Cell::plain(subscription.config_count.to_string()),
                Cell::plain(output::truncate(
                    subscription.name.as_deref().unwrap_or("-"),
                    24,
                )),
                Cell::plain(output::truncate(
                    subscription.source_url.as_deref().unwrap_or("-"),
                    56,
                )),
            ]
        })
        .collect::<Vec<_>>();

    output::format_table(&columns, &rows, output::color_enabled())
}

fn format_subscription_tsv(subscriptions: &[SubscriptionRecord]) -> String {
    let mut lines = Vec::with_capacity(subscriptions.len() + 1);
    lines.push("ref\tkind\tconfig_count\tname\tsource".to_string());
    for subscription in subscriptions {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}",
            subscription.r#ref,
            subscription.source_kind,
            subscription.config_count,
            tsv_cell(subscription.name.as_deref()),
            tsv_cell(subscription.source_url.as_deref()),
        ));
    }
    lines.join("\n")
}

fn format_config_flags(is_enabled: bool, is_active: bool, is_deleted: bool) -> String {
    let mut flags = Vec::new();

    if is_deleted {
        flags.push("deleted");
    }
    if is_enabled {
        flags.push("enabled");
    } else {
        flags.push("disabled");
    }
    if is_active {
        flags.push("active");
    }

    flags.join(",")
}

fn config_style(config: &ConfigRecord) -> Style {
    if config.is_deleted {
        Style::Red
    } else if config.is_active {
        Style::Green
    } else if config.is_enabled {
        Style::Cyan
    } else {
        Style::Dim
    }
}

fn config_json(config: &ConfigRecord, subscription_refs: &HashMap<i64, &str>) -> serde_json::Value {
    serde_json::json!({
        "ref": &config.r#ref,
        "subscription_ref": config
            .subscription_id
            .and_then(|id| subscription_refs.get(&id).copied()),
        "protocol": config.protocol,
        "address": config.address,
        "port": config.port,
        "name": config.name,
        "network": config.network,
        "tls": config.tls,
        "is_active": config.is_active,
        "is_enabled": config.is_enabled,
        "is_deleted": config.is_deleted,
        "deleted_at": config.deleted_at,
        "imported_at": config.imported_at,
        "created_at": config.created_at,
        "updated_at": config.updated_at,
    })
}

pub(crate) fn subscription_json(subscription: &SubscriptionRecord) -> serde_json::Value {
    serde_json::json!({
        "ref": &subscription.r#ref,
        "source_kind": subscription.source_kind,
        "source_url": subscription.source_url,
        "name": subscription.name,
        "created_at": subscription.created_at,
        "updated_at": subscription.updated_at,
        "config_count": subscription.config_count,
    })
}

fn tsv_cell(value: Option<&str>) -> String {
    value.unwrap_or_default().replace(['\t', '\r', '\n'], " ")
}

fn subscription_ref_cell(
    subscription_id: Option<i64>,
    subscription_refs: &HashMap<i64, &str>,
) -> String {
    subscription_id
        .and_then(|id| subscription_refs.get(&id).copied())
        .map(short_ref)
        .unwrap_or("-")
        .to_string()
}

fn subscription_ref_tsv_cell(
    subscription_id: Option<i64>,
    subscription_refs: &HashMap<i64, &str>,
) -> String {
    subscription_id
        .and_then(|id| subscription_refs.get(&id).copied())
        .unwrap_or_default()
        .to_string()
}

async fn build_config_list_filter(
    context: &AppContext,
    args: &ListConfigsArgs,
) -> crate::app::Result<ConfigListFilter> {
    let subscription_id = match &args.subscription {
        Some(raw) => Some(resolve_subscription_id(context, raw).await?),
        None => None,
    };

    Ok(ConfigListFilter {
        only_enabled: args.enabled_only,
        only_active: args.active_only,
        only_deleted: args.deleted_only,
        include_deleted: args.include_deleted,
        subscription_id,
        protocol: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_outputs_include_refs() {
        let config = config_record("abcdef123456");
        let subscriptions = HashMap::from([(2, "123456abcdef")]);

        let table = format_config_table(std::slice::from_ref(&config), &subscriptions);
        let tsv = format_config_tsv(std::slice::from_ref(&config), &subscriptions);
        let json = config_json(&config, &subscriptions);

        assert!(table.contains("REF"));
        assert!(table.contains("abcdef12"));
        assert!(table.contains("123456ab"));
        assert!(tsv.starts_with("ref\tsubscription_ref\t"));
        assert!(!tsv.starts_with("ref\tid\t"));
        assert_eq!(json["ref"], "abcdef123456");
        assert_eq!(json["subscription_ref"], "123456abcdef");
        assert!(json.get("id").is_none());
        assert!(json.get("subscription_id").is_none());
    }

    #[test]
    fn subscription_outputs_include_refs() {
        let subscription = SubscriptionRecord {
            id: 2,
            r#ref: "123456abcdef".to_string(),
            source_kind: "url".to_string(),
            source_url: Some("https://example.com/sub".to_string()),
            name: Some("main".to_string()),
            created_at: "created".to_string(),
            updated_at: "updated".to_string(),
            config_count: 3,
        };

        let table = format_subscription_table(std::slice::from_ref(&subscription));
        let tsv = format_subscription_tsv(std::slice::from_ref(&subscription));
        let json = subscription_json(&subscription);

        assert!(table.contains("REF"));
        assert!(table.contains("123456ab"));
        assert!(tsv.starts_with("ref\tkind\t"));
        assert_eq!(json["ref"], "123456abcdef");
        assert!(json.get("id").is_none());
    }

    fn config_record(value_ref: &str) -> ConfigRecord {
        ConfigRecord {
            id: 1,
            r#ref: value_ref.to_string(),
            subscription_id: Some(2),
            dedup_key: "key".to_string(),
            protocol: "vless".to_string(),
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: Some("uuid".to_string()),
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: Some("tls".to_string()),
            sni: None,
            host: None,
            path: None,
            name: Some("main".to_string()),
            raw_config: "vless://uuid@example.com:443#main".to_string(),
            is_active: true,
            is_enabled: true,
            is_deleted: false,
            deleted_at: None,
            imported_at: "imported".to_string(),
            created_at: "created".to_string(),
            updated_at: "updated".to_string(),
        }
    }
}
