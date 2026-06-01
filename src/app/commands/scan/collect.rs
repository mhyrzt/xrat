use std::collections::BTreeSet;

use crate::cli::ScanArgs;

pub(super) fn collect_ips(args: &ScanArgs) -> crate::app::Result<Vec<String>> {
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
