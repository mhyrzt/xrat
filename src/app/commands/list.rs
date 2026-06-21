use crate::app::commands::output::{self, Align, Cell, Column, Style};
use crate::app::commands::resolve::resolve_subscription_id;
use crate::app::context::AppContext;
use crate::cli::{ListArgs, ListConfigsArgs, ListFormat, ListSubscriptionsArgs, ListTarget};
use crate::db::{ConfigListFilter, ConfigRecord, ConfigWithLatestTest, SubscriptionRecord};
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
    let mut configs = context.db.list_configs_with_latest_tests(&filter).await?;
    enrich_config_locations(context, &mut configs).await;
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
        format_configs(
            &configs,
            &subscription_refs,
            filters.format,
            Some(&context.app_config.testing),
        )?
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
    configs: &[ConfigWithLatestTest],
    subscription_refs: &HashMap<i64, &str>,
    format: ListFormat,
    settings: Option<&crate::app::config::TestingSettings>,
) -> crate::app::Result<String> {
    match format {
        ListFormat::Table => Ok(format_config_table(configs, subscription_refs, settings)),
        ListFormat::Tsv => Ok(format_config_tsv(configs, subscription_refs)),
        ListFormat::Json => Ok(serde_json::to_string_pretty(
            &configs
                .iter()
                .map(|config| config_json(config, subscription_refs))
                .collect::<Vec<_>>(),
        )?),
    }
}

fn format_config_table(
    configs: &[ConfigWithLatestTest],
    subscription_refs: &HashMap<i64, &str>,
    settings: Option<&crate::app::config::TestingSettings>,
) -> String {
    let metric_columns = MetricColumns::for_configs(configs, settings);
    let mut columns = vec![
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
    ];
    metric_columns.push_columns(&mut columns);
    columns.extend([Column {
        header: "NAME",
        align: Align::Left,
    }]);
    let rows = configs
        .iter()
        .map(|row| {
            let config = &row.config;
            let mut cells = vec![
                Cell::plain(short_ref(&config.r#ref)),
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
            ];
            metric_columns.push_cells(row, &mut cells);
            cells.push(Cell::plain(output::truncate(
                config.name.as_deref().unwrap_or("-"),
                32,
            )));
            cells
        })
        .collect::<Vec<_>>();

    output::format_table(&columns, &rows, output::color_enabled())
}

fn format_config_tsv(
    configs: &[ConfigWithLatestTest],
    subscription_refs: &HashMap<i64, &str>,
) -> String {
    let mut lines = Vec::with_capacity(configs.len() + 1);
    lines.push("ref\tsubscription_ref\tstatus\tprotocol\taddress\tport\ticmp_ms\ttcp_ms\treal_delay_ms\tdownload_mbps\tupload_mbps\tdial_endpoint_country\tdial_endpoint_location\tdial_endpoint_asn\tdial_endpoint_fronting\tname".to_string());
    for row in configs {
        let config = &row.config;
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            config.r#ref,
            subscription_ref_tsv_cell(config.subscription_id, subscription_refs),
            format_config_flags(config.is_enabled, config.is_active, config.is_deleted),
            config.protocol,
            config.address,
            config.port,
            optional_i64(row.icmp_ms),
            optional_i64(row.tcp_ms),
            optional_i64(row.real_delay_ms),
            optional_f64(row.download_mbps),
            optional_f64(row.upload_mbps),
            tsv_cell(row.dial_endpoint_country.as_deref()),
            tsv_cell(row.dial_endpoint_location.as_deref()),
            tsv_cell(row.dial_endpoint_asn.as_deref()),
            tsv_cell(row.dial_endpoint_fronting.as_deref()),
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
        Column {
            header: "UPDATED AT",
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
                Cell::plain(subscription.updated_at.clone()),
            ]
        })
        .collect::<Vec<_>>();

    output::format_table(&columns, &rows, output::color_enabled())
}

fn format_subscription_tsv(subscriptions: &[SubscriptionRecord]) -> String {
    let mut lines = Vec::with_capacity(subscriptions.len() + 1);
    lines.push("ref\tkind\tconfig_count\tname\tsource\tupdated_at".to_string());
    for subscription in subscriptions {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            subscription.r#ref,
            subscription.source_kind,
            subscription.config_count,
            tsv_cell(subscription.name.as_deref()),
            tsv_cell(subscription.source_url.as_deref()),
            subscription.updated_at,
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

fn config_json(
    row: &ConfigWithLatestTest,
    subscription_refs: &HashMap<i64, &str>,
) -> serde_json::Value {
    let config = &row.config;
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
        "latest_test": {
            "id": row.test_id,
            "icmp_ok": row.icmp_ok,
            "icmp_ms": row.icmp_ms,
            "tcp_ok": row.tcp_ok,
            "tcp_ms": row.tcp_ms,
            "real_delay_ok": row.real_delay_ok,
            "real_delay_ms": row.real_delay_ms,
            "download_mbps": row.download_mbps,
            "upload_mbps": row.upload_mbps,
            "connect_ms": row.connect_ms,
            "ttfb_ms": row.ttfb_ms,
            "http_status": row.http_status,
            "dial_endpoint_location": row.dial_endpoint_location,
            "dial_endpoint_country": row.dial_endpoint_country,
            "dial_endpoint_asn": row.dial_endpoint_asn,
            "dial_endpoint_geoip_source": row.dial_endpoint_geoip_source,
            "dial_endpoint_fronting": row.dial_endpoint_fronting,
            "failure_kind": row.failure_kind,
            "failure_reason": row.failure_reason,
            "tested_at": row.tested_at,
        },
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

fn optional_i64(value: Option<i64>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn optional_f64(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.2}")).unwrap_or_default()
}

fn ms_label(value: Option<i64>) -> String {
    value
        .map(|value| format!("{value}ms"))
        .unwrap_or_else(|| "-".to_string())
}

fn mbps_label(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.1}"))
        .unwrap_or_else(|| "-".to_string())
}

fn location_cell(value: Option<&str>, max_width: usize) -> String {
    output::truncate(value.unwrap_or("-"), max_width)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct MetricColumns {
    icmp: bool,
    tcp: bool,
    real_delay: bool,
    download: bool,
    upload: bool,
    country: bool,
    location: bool,
    asn: bool,
}

impl MetricColumns {
    fn for_configs(
        configs: &[ConfigWithLatestTest],
        settings: Option<&crate::app::config::TestingSettings>,
    ) -> Self {
        if let Some(settings) = settings {
            return Self {
                icmp: settings.icmp.enabled,
                tcp: settings.tcp.enabled,
                real_delay: settings.real_delay.enabled,
                download: settings.download.enabled,
                upload: false,
                country: settings.geoip.enabled,
                location: settings.geoip.enabled,
                asn: settings.geoip.enabled,
            };
        }

        Self {
            icmp: configs.iter().any(|row| row.icmp_ms.is_some()),
            tcp: configs.iter().any(|row| row.tcp_ms.is_some()),
            real_delay: configs.iter().any(|row| row.real_delay_ms.is_some()),
            download: configs.iter().any(|row| row.download_mbps.is_some()),
            upload: configs.iter().any(|row| row.upload_mbps.is_some()),
            country: configs
                .iter()
                .any(|row| row.dial_endpoint_country.is_some()),
            location: configs
                .iter()
                .any(|row| row.dial_endpoint_location.is_some()),
            asn: configs.iter().any(|row| row.dial_endpoint_asn.is_some()),
        }
    }

    fn push_columns(self, columns: &mut Vec<Column>) {
        if self.icmp {
            columns.push(Column {
                header: "ICMP",
                align: Align::Right,
            });
        }
        if self.tcp {
            columns.push(Column {
                header: "TCP",
                align: Align::Right,
            });
        }
        if self.real_delay {
            columns.push(Column {
                header: "REAL",
                align: Align::Right,
            });
        }
        if self.download {
            columns.push(Column {
                header: "DOWN",
                align: Align::Right,
            });
        }
        if self.upload {
            columns.push(Column {
                header: "UP",
                align: Align::Right,
            });
        }
        if self.country {
            columns.push(Column {
                header: "COUNTRY",
                align: Align::Left,
            });
        }
        if self.location {
            columns.push(Column {
                header: "CITY",
                align: Align::Left,
            });
        }
        if self.asn {
            columns.push(Column {
                header: "ASN",
                align: Align::Left,
            });
        }
    }

    fn push_cells(self, row: &ConfigWithLatestTest, cells: &mut Vec<Cell>) {
        if self.icmp {
            cells.push(Cell::plain(ms_label(row.icmp_ms)));
        }
        if self.tcp {
            cells.push(Cell::plain(ms_label(row.tcp_ms)));
        }
        if self.real_delay {
            cells.push(Cell::plain(ms_label(row.real_delay_ms)));
        }
        if self.download {
            cells.push(Cell::plain(mbps_label(row.download_mbps)));
        }
        if self.upload {
            cells.push(Cell::plain(mbps_label(row.upload_mbps)));
        }
        if self.country {
            cells.push(Cell::plain(location_cell(
                row.dial_endpoint_country.as_deref(),
                10,
            )));
        }
        if self.location {
            cells.push(Cell::plain(location_cell(
                row.dial_endpoint_location.as_deref(),
                24,
            )));
        }
        if self.asn {
            cells.push(Cell::plain(location_cell(
                row.dial_endpoint_asn.as_deref(),
                24,
            )));
        }
    }
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

async fn enrich_config_locations(context: &AppContext, configs: &mut [ConfigWithLatestTest]) {
    if !context.app_config.testing.geoip.enabled
        || !configs.iter().any(|row| row.needs_location_enrichment())
    {
        return;
    }

    let Ok(lookup) =
        crate::support::geoip::build_lookup_chain(&context.app_config, &context.runtime_paths)
    else {
        return;
    };

    for row in configs
        .iter_mut()
        .filter(|row| row.needs_location_enrichment())
    {
        let meta =
            crate::support::geoip::enrich_address(&row.config.address, lookup.as_ref()).await;
        if !meta.has_lookup_metadata() {
            continue;
        }
        if let Some(location) = meta.location {
            row.dial_endpoint_location = Some(location);
        }
        if let Some(country) = meta.country {
            row.dial_endpoint_country = Some(country);
        }
        if let Some(asn) = meta.asn {
            row.dial_endpoint_asn = Some(asn);
        }
    }
}

trait ConfigLocationEnrichment {
    fn needs_location_enrichment(&self) -> bool;
}

impl ConfigLocationEnrichment for ConfigWithLatestTest {
    fn needs_location_enrichment(&self) -> bool {
        self.dial_endpoint_location.is_none()
            || self.dial_endpoint_country.is_none()
            || self.dial_endpoint_asn.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_outputs_include_refs() {
        let config = config_row("abcdef123456");
        let subscriptions = HashMap::from([(2, "123456abcdef")]);

        let table = format_config_table(std::slice::from_ref(&config), &subscriptions, None);
        let tsv = format_config_tsv(std::slice::from_ref(&config), &subscriptions);
        let json = config_json(&config, &subscriptions);

        assert!(table.contains("REF"));
        assert!(table.contains("ICMP"));
        assert!(table.contains("42ms"));
        assert!(table.contains("COUNTRY"));
        assert!(table.contains("NL"));
        assert!(table.contains("abcdef12"));
        assert!(table.contains("123456ab"));
        assert!(
            tsv.starts_with("ref\tsubscription_ref\tstatus\tprotocol\taddress\tport\ticmp_ms\t")
        );
        assert!(!tsv.starts_with("ref\tid\t"));
        assert!(tsv.contains("\t42\t20\t100\t25.50\t5.75\tNL\tNL/Amsterdam\tAS60781 LeaseWeb\t"));
        assert_eq!(json["ref"], "abcdef123456");
        assert_eq!(json["subscription_ref"], "123456abcdef");
        assert_eq!(json["latest_test"]["icmp_ms"], 42);
        assert_eq!(json["latest_test"]["dial_endpoint_country"], "NL");
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
        assert!(table.contains("UPDATED AT"));
        assert!(table.contains("updated"));
        assert!(tsv.starts_with("ref\tkind\t"));
        assert_eq!(json["ref"], "123456abcdef");
        assert!(json.get("id").is_none());
    }

    #[test]
    fn config_table_uses_enabled_settings_for_metric_columns() {
        let config = config_row("abcdef123456");
        let subscriptions = HashMap::from([(2, "123456abcdef")]);
        let mut settings = crate::app::config::TestingSettings::default();
        settings.icmp.enabled = false;
        settings.tcp.enabled = true;
        settings.real_delay.enabled = true;
        settings.download.enabled = false;
        settings.geoip.enabled = false;

        let table = format_config_table(
            std::slice::from_ref(&config),
            &subscriptions,
            Some(&settings),
        );

        assert!(!table.contains("ICMP"));
        assert!(table.contains("TCP"));
        assert!(table.contains("REAL"));
        assert!(!table.contains("DOWN"));
        assert!(!table.contains("COUNTRY"));
    }

    fn config_row(value_ref: &str) -> ConfigWithLatestTest {
        ConfigWithLatestTest {
            config: ConfigRecord {
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
            },
            test_id: Some(9),
            icmp_ok: Some(true),
            icmp_ms: Some(42),
            tcp_ok: Some(true),
            tcp_ms: Some(20),
            real_delay_ok: Some(true),
            real_delay_ms: Some(100),
            download_mbps: Some(25.5),
            upload_mbps: Some(5.75),
            connect_ms: Some(20),
            ttfb_ms: Some(80),
            http_status: Some(204),
            dial_endpoint_location: Some("NL/Amsterdam".to_string()),
            dial_endpoint_country: Some("NL".to_string()),
            dial_endpoint_asn: Some("AS60781 LeaseWeb".to_string()),
            dial_endpoint_geoip_source: None,
            dial_endpoint_fronting: None,
            failure_kind: None,
            failure_reason: None,
            tested_at: Some("tested".to_string()),
        }
    }
}
