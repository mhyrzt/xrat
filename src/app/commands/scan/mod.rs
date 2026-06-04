mod collect;

use std::time::Duration;

use crate::app::AppError;
use crate::app::commands::output::{self, Align, Cell, Column, Style};
use crate::app::context::AppContext;
use crate::cli::{ListFormat, ScanArgs};
use crate::db::{CfScanResultRecord, CfScanResultUpsert};
use crate::prober::tcp_check;

pub async fn run(context: &AppContext, args: &ScanArgs) -> crate::app::Result<()> {
    if let Some(limit) = args.history {
        let rows = context.db.list_cf_scan_history(limit.max(1)).await?;
        if rows.is_empty() {
            println!("{}", output::empty_message("No scanner history found."));
            return Ok(());
        }
        println!("{}", format_history(&rows, args.format)?);
        return Ok(());
    }

    let ips = collect::collect_ips(args)?;
    if ips.is_empty() {
        return Err(AppError::InvalidArgument(
            "scan requires at least one IP via --ips or --file".to_string(),
        ));
    }

    let timeout = Duration::from_millis(args.timeout_ms.max(100));
    let mut results = Vec::with_capacity(ips.len());

    for ip in ips {
        let probe = tcp_check(&ip, args.port, timeout).await;
        results.push(CfScanResultUpsert {
            ip,
            latency_ms: probe.latency_ms.map(i64::from),
            download_mbps: None,
            upload_mbps: None,
            error: probe.failure_reason,
        });
    }

    context.db.upsert_cf_scan_results(&results).await?;
    println!(
        "{}",
        output::success(
            format!("Persisted {} scanner rows.", results.len()),
            output::color_enabled()
        )
    );
    Ok(())
}

fn format_history(rows: &[CfScanResultRecord], format: ListFormat) -> crate::app::Result<String> {
    match format {
        ListFormat::Table => Ok(format_history_table(rows)),
        ListFormat::Tsv => Ok(format_history_tsv(rows)),
        ListFormat::Json => Ok(serde_json::to_string_pretty(
            &rows.iter().map(history_json).collect::<Vec<_>>(),
        )?),
    }
}

fn format_history_table(rows: &[CfScanResultRecord]) -> String {
    let columns = [
        Column {
            header: "IP",
            align: Align::Left,
        },
        Column {
            header: "LATENCY",
            align: Align::Right,
        },
        Column {
            header: "DOWN",
            align: Align::Right,
        },
        Column {
            header: "UP",
            align: Align::Right,
        },
        Column {
            header: "STATUS",
            align: Align::Left,
        },
        Column {
            header: "SCANNED",
            align: Align::Left,
        },
    ];
    let table_rows = rows
        .iter()
        .map(|row| {
            let failed = row.error.is_some();
            vec![
                Cell::plain(row.ip.clone()),
                Cell::plain(
                    row.latency_ms
                        .map(|value| format!("{value}ms"))
                        .unwrap_or_default(),
                ),
                Cell::plain(
                    row.download_mbps
                        .map(|value| format!("{value:.2}"))
                        .unwrap_or_default(),
                ),
                Cell::plain(
                    row.upload_mbps
                        .map(|value| format!("{value:.2}"))
                        .unwrap_or_default(),
                ),
                Cell::styled(
                    row.error.as_deref().unwrap_or("ok"),
                    if failed { Style::Red } else { Style::Green },
                ),
                Cell::plain(row.last_scanned_at.clone()),
            ]
        })
        .collect::<Vec<_>>();

    output::format_table(&columns, &table_rows, output::color_enabled())
}

fn format_history_tsv(rows: &[CfScanResultRecord]) -> String {
    let mut lines = Vec::with_capacity(rows.len() + 1);
    lines.push("ip\tlatency_ms\tdownload_mbps\tupload_mbps\terror\tlast_scanned_at".to_string());
    for row in rows {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            row.ip,
            row.latency_ms
                .map(|value| value.to_string())
                .unwrap_or_default(),
            row.download_mbps
                .map(|value| format!("{value:.2}"))
                .unwrap_or_default(),
            row.upload_mbps
                .map(|value| format!("{value:.2}"))
                .unwrap_or_default(),
            row.error
                .as_deref()
                .unwrap_or_default()
                .replace(['\t', '\r', '\n'], " "),
            row.last_scanned_at,
        ));
    }
    lines.join("\n")
}

fn history_json(row: &CfScanResultRecord) -> serde_json::Value {
    serde_json::json!({
        "id": row.id,
        "ip": row.ip,
        "latency_ms": row.latency_ms,
        "download_mbps": row.download_mbps,
        "upload_mbps": row.upload_mbps,
        "error": row.error,
        "last_scanned_at": row.last_scanned_at,
    })
}
