use std::collections::BTreeSet;
use std::time::Duration;

use crate::app::AppError;
use crate::app::context::AppContext;
use crate::cli::ScanArgs;
use crate::db::CfScanResultUpsert;
use crate::prober::tcp_check;

pub async fn run(context: &AppContext, args: &ScanArgs) -> crate::app::Result<()> {
    if let Some(limit) = args.history {
        let rows = context.db.list_cf_scan_history(limit.max(1)).await?;
        if rows.is_empty() {
            println!("No scanner history found.");
            return Ok(());
        }
        for row in rows {
            println!(
                "{} latency_ms={} download_mbps={} upload_mbps={} error={} at={}",
                row.ip,
                row.latency_ms
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                row.download_mbps
                    .map(|v| format!("{v:.2}"))
                    .unwrap_or_else(|| "-".to_string()),
                row.upload_mbps
                    .map(|v| format!("{v:.2}"))
                    .unwrap_or_else(|| "-".to_string()),
                row.error.unwrap_or_else(|| "-".to_string()),
                row.last_scanned_at,
            );
        }
        return Ok(());
    }

    let ips = collect_ips(args)?;
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
    println!("Persisted {} scanner rows.", results.len());
    Ok(())
}

fn collect_ips(args: &ScanArgs) -> crate::app::Result<Vec<String>> {
    let mut dedup = BTreeSet::new();
    for ip in &args.ips {
        let ip = ip.trim();
        if !ip.is_empty() {
            dedup.insert(ip.to_string());
        }
    }

    if let Some(path) = &args.file {
        let input = std::fs::read_to_string(path)?;
        for line in input.lines() {
            let ip = line.trim();
            if !ip.is_empty() {
                dedup.insert(ip.to_string());
            }
        }
    }

    Ok(dedup.into_iter().collect())
}
