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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_ips_from_args_with_dedup() {
        let args = ScanArgs {
            ips: vec![
                "1.2.3.4".to_string(),
                "5.6.7.8".to_string(),
                "1.2.3.4".to_string(),
            ],
            file: None,
            port: 443,
            timeout_ms: 1000,
            history: None,
        };

        let ips = collect_ips(&args).expect("should collect");
        assert_eq!(ips.len(), 2);
        assert!(ips.contains(&"1.2.3.4".to_string()));
        assert!(ips.contains(&"5.6.7.8".to_string()));
    }

    #[test]
    fn trims_whitespace_from_ips() {
        let args = ScanArgs {
            ips: vec!["  1.2.3.4  ".to_string(), "\t5.6.7.8\n".to_string()],
            file: None,
            port: 443,
            timeout_ms: 1000,
            history: None,
        };

        let ips = collect_ips(&args).expect("should collect");
        assert_eq!(ips.len(), 2);
        assert!(ips.contains(&"1.2.3.4".to_string()));
        assert!(ips.contains(&"5.6.7.8".to_string()));
    }

    #[test]
    fn filters_empty_ips() {
        let args = ScanArgs {
            ips: vec![
                "1.2.3.4".to_string(),
                "".to_string(),
                "   ".to_string(),
                "5.6.7.8".to_string(),
            ],
            file: None,
            port: 443,
            timeout_ms: 1000,
            history: None,
        };

        let ips = collect_ips(&args).expect("should collect");
        assert_eq!(ips.len(), 2);
    }

    #[test]
    fn returns_sorted_ips_via_btree() {
        let args = ScanArgs {
            ips: vec![
                "9.9.9.9".to_string(),
                "1.1.1.1".to_string(),
                "5.5.5.5".to_string(),
            ],
            file: None,
            port: 443,
            timeout_ms: 1000,
            history: None,
        };

        let ips = collect_ips(&args).expect("should collect");
        assert_eq!(ips, vec!["1.1.1.1", "5.5.5.5", "9.9.9.9"]);
    }

    #[test]
    fn reads_ips_from_file() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file_path = dir.path().join("ips.txt");
        std::fs::write(&file_path, "1.2.3.4\n5.6.7.8\n\n9.9.9.9\n").expect("write");

        let args = ScanArgs {
            ips: vec![],
            file: Some(file_path),
            port: 443,
            timeout_ms: 1000,
            history: None,
        };

        let ips = collect_ips(&args).expect("should collect from file");
        assert_eq!(ips.len(), 3);
    }

    #[test]
    fn merges_ips_from_args_and_file() {
        let dir = tempfile::tempdir().expect("temp dir");
        let file_path = dir.path().join("ips.txt");
        std::fs::write(&file_path, "1.2.3.4\n5.6.7.8\n").expect("write");

        let args = ScanArgs {
            ips: vec!["1.2.3.4".to_string(), "9.9.9.9".to_string()],
            file: Some(file_path),
            port: 443,
            timeout_ms: 1000,
            history: None,
        };

        let ips = collect_ips(&args).expect("should merge");
        assert_eq!(ips.len(), 3);
    }

    #[test]
    fn returns_empty_when_no_ips_provided() {
        let args = ScanArgs {
            ips: vec![],
            file: None,
            port: 443,
            timeout_ms: 1000,
            history: None,
        };

        let ips = collect_ips(&args).expect("should return empty");
        assert!(ips.is_empty());
    }
}
