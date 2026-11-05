use super::prelude::*;
use crate::app::commands::output;

pub(crate) fn format_configs(
    configs: &[ConfigDetail],
    subscription_refs: &HashMap<SubscriptionId, &str>,
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

pub(crate) fn format_config_table(
    configs: &[ConfigDetail],
    subscription_refs: &HashMap<SubscriptionId, &str>,
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
            let config = &row.summary;
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
                Cell::plain(if config.protocol == xrat_db::XRAY_PROFILE_PROTOCOL {
                    "-".into()
                } else {
                    config.port.to_string()
                }),
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

pub(crate) fn format_config_tsv(
    configs: &[ConfigDetail],
    subscription_refs: &HashMap<SubscriptionId, &str>,
) -> String {
    let mut lines = Vec::with_capacity(configs.len() + 1);
    lines.push("ref\tsubscription_ref\tstatus\tprotocol\taddress\tport\ticmp_ms\ttcp_ms\treal_delay_ms\tdownload_mbps\tupload_mbps\tdial_endpoint_country\tdial_endpoint_location\tdial_endpoint_asn\tdial_endpoint_fronting\tname".to_string());
    for row in configs {
        let config = &row.summary;
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            config.r#ref,
            subscription_ref_tsv_cell(config.subscription_id, subscription_refs),
            format_config_flags(config.is_enabled, config.is_active, config.is_deleted),
            config.protocol,
            config.address,
            config.port,
            optional_i64(row.summary.test().icmp_ms),
            optional_i64(row.summary.test().tcp_ms),
            optional_i64(row.summary.test().real_delay_ms),
            optional_f64(row.summary.test().download_mbps),
            optional_f64(row.summary.test().upload_mbps),
            tsv_cell(row.endpoint_location.country.as_deref()),
            tsv_cell(row.endpoint_location.location.as_deref()),
            tsv_cell(row.endpoint_location.asn.as_deref()),
            tsv_cell(row.endpoint_location.fronting.as_deref()),
            tsv_cell(config.name.as_deref()),
        ));
    }
    lines.join("\n")
}

pub(crate) fn format_config_flags(is_enabled: bool, is_active: bool, is_deleted: bool) -> String {
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

pub(crate) fn config_style(config: &ConfigSummary) -> Style {
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

pub(crate) fn config_json(
    row: &ConfigDetail,
    subscription_refs: &HashMap<SubscriptionId, &str>,
) -> serde_json::Value {
    let config = &row.summary;
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
            "id": row.summary.test().id,
            "icmp_ok": row.summary.test().icmp_ok,
            "icmp_ms": row.summary.test().icmp_ms,
            "tcp_ok": row.summary.test().tcp_ok,
            "tcp_ms": row.summary.test().tcp_ms,
            "real_delay_ok": row.summary.test().real_delay_ok,
            "real_delay_ms": row.summary.test().real_delay_ms,
            "download_mbps": row.summary.test().download_mbps,
            "upload_mbps": row.summary.test().upload_mbps,
            "connect_ms": row.summary.test().connect_ms,
            "ttfb_ms": row.summary.test().ttfb_ms,
            "http_status": row.summary.test().http_status,
            "dial_endpoint_location": row.endpoint_location.location,
            "dial_endpoint_country": row.endpoint_location.country,
            "dial_endpoint_asn": row.endpoint_location.asn,
            "dial_endpoint_geoip_source": row.endpoint_location.geoip_source,
            "dial_endpoint_fronting": row.endpoint_location.fronting,
            "failure_kind": row.summary.test().failure_kind,
            "failure_reason": row.summary.test().failure_reason,
            "tested_at": row.summary.test().tested_at,
        },
        "deleted_at": row.deleted_at,
        "imported_at": row.imported_at,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
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

pub(crate) fn tsv_cell(value: Option<&str>) -> String {
    value.unwrap_or_default().replace(['\t', '\r', '\n'], " ")
}

pub(crate) fn optional_i64(value: Option<i64>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

pub(crate) fn optional_f64(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.2}")).unwrap_or_default()
}

pub(crate) fn ms_label(value: Option<i64>) -> String {
    value
        .map(|value| format!("{value}ms"))
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) fn mbps_label(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.1}"))
        .unwrap_or_else(|| "-".to_string())
}

pub(crate) fn location_cell(value: Option<&str>, max_width: usize) -> String {
    output::truncate(value.unwrap_or("-"), max_width)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MetricColumns {
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
        configs: &[ConfigDetail],
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
            icmp: configs
                .iter()
                .any(|row| row.summary.test().icmp_ms.is_some()),
            tcp: configs
                .iter()
                .any(|row| row.summary.test().tcp_ms.is_some()),
            real_delay: configs
                .iter()
                .any(|row| row.summary.test().real_delay_ms.is_some()),
            download: configs
                .iter()
                .any(|row| row.summary.test().download_mbps.is_some()),
            upload: configs
                .iter()
                .any(|row| row.summary.test().upload_mbps.is_some()),
            country: configs
                .iter()
                .any(|row| row.endpoint_location.country.is_some()),
            location: configs
                .iter()
                .any(|row| row.endpoint_location.location.is_some()),
            asn: configs
                .iter()
                .any(|row| row.endpoint_location.asn.is_some()),
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

    fn push_cells(self, row: &ConfigDetail, cells: &mut Vec<Cell>) {
        if self.icmp {
            cells.push(Cell::plain(ms_label(row.summary.test().icmp_ms)));
        }
        if self.tcp {
            cells.push(Cell::plain(ms_label(row.summary.test().tcp_ms)));
        }
        if self.real_delay {
            cells.push(Cell::plain(ms_label(row.summary.test().real_delay_ms)));
        }
        if self.download {
            cells.push(Cell::plain(mbps_label(row.summary.test().download_mbps)));
        }
        if self.upload {
            cells.push(Cell::plain(mbps_label(row.summary.test().upload_mbps)));
        }
        if self.country {
            cells.push(Cell::plain(location_cell(
                row.endpoint_location.country.as_deref(),
                10,
            )));
        }
        if self.location {
            cells.push(Cell::plain(location_cell(
                row.endpoint_location.location.as_deref(),
                24,
            )));
        }
        if self.asn {
            cells.push(Cell::plain(location_cell(
                row.endpoint_location.asn.as_deref(),
                24,
            )));
        }
    }
}

pub(crate) fn subscription_ref_cell(
    subscription_id: Option<SubscriptionId>,
    subscription_refs: &HashMap<SubscriptionId, &str>,
) -> String {
    subscription_id
        .and_then(|id| subscription_refs.get(&id).copied())
        .map(short_ref)
        .unwrap_or("-")
        .to_string()
}

pub(crate) fn subscription_ref_tsv_cell(
    subscription_id: Option<SubscriptionId>,
    subscription_refs: &HashMap<SubscriptionId, &str>,
) -> String {
    subscription_id
        .and_then(|id| subscription_refs.get(&id).copied())
        .unwrap_or_default()
        .to_string()
}
