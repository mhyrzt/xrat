use std::fs;
use std::path::Path;

use crate::app::AppError;
use crate::app::commands::output::{self, Align, Cell, Column, Style};
use crate::app::context::AppContext;
use crate::cli::GeoIpStatusArgs;

use super::{SUPPORTED_EDITIONS, mmdb_file_name, mmdb_file_path, resolve_mmdb_target_dir};

pub fn run(context: &AppContext, args: &GeoIpStatusArgs) -> crate::app::Result<()> {
    let dir = resolve_mmdb_target_dir(context, args.output.as_ref());
    let statuses = collect_statuses(&dir)?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "mmdb_dir": dir,
                "editions": statuses.iter().map(status_json).collect::<Vec<_>>(),
            }))?
        );
    } else {
        println!("{}", format_status_table(&dir, &statuses));
    }

    if args.strict && statuses.iter().any(|status| status.is_missing()) {
        return Err(AppError::InvalidArgument(
            "one or more MMDB editions are missing".to_string(),
        ));
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EditionStatus {
    file_name: &'static str,
    state: &'static str,
    size_bytes: Option<u64>,
}

impl EditionStatus {
    fn is_missing(&self) -> bool {
        self.state == "missing"
    }
}

fn collect_statuses(dir: &Path) -> crate::app::Result<Vec<EditionStatus>> {
    let mut statuses = Vec::with_capacity(SUPPORTED_EDITIONS.len());

    for edition in SUPPORTED_EDITIONS {
        let file_name = mmdb_file_name(edition);
        let path = mmdb_file_path(dir, edition);
        let metadata = match fs::metadata(&path) {
            Ok(metadata) => Some(metadata),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
            Err(err) => return Err(err.into()),
        };

        statuses.push(match metadata {
            Some(metadata) => EditionStatus {
                file_name,
                state: "present",
                size_bytes: Some(metadata.len()),
            },
            None => EditionStatus {
                file_name,
                state: "missing",
                size_bytes: None,
            },
        });
    }

    Ok(statuses)
}

fn format_status_table(dir: &Path, statuses: &[EditionStatus]) -> String {
    let mut lines = Vec::new();
    lines.push(output::format_kv(
        Some("GeoIP"),
        &[("mmdb dir", dir.display().to_string())],
        output::color_enabled(),
    ));
    lines.push(String::new());
    lines.push(output::format_table(
        &[
            Column {
                header: "EDITION",
                align: Align::Left,
            },
            Column {
                header: "STATE",
                align: Align::Left,
            },
            Column {
                header: "SIZE",
                align: Align::Right,
            },
        ],
        &statuses
            .iter()
            .map(|status| {
                vec![
                    Cell::plain(status.file_name),
                    Cell::styled(
                        status.state,
                        if status.is_missing() {
                            Style::Yellow
                        } else {
                            Style::Green
                        },
                    ),
                    Cell::plain(
                        status
                            .size_bytes
                            .map(format_bytes)
                            .unwrap_or_else(|| "-".to_string()),
                    ),
                ]
            })
            .collect::<Vec<_>>(),
        output::color_enabled(),
    ));
    lines.join("\n")
}

fn status_json(status: &EditionStatus) -> serde_json::Value {
    serde_json::json!({
        "file_name": status.file_name,
        "state": status.state,
        "size_bytes": status.size_bytes,
    })
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];

    let mut value = bytes as f64;
    let mut unit = UNITS[0];
    for next_unit in UNITS.iter().skip(1) {
        if value < 1024.0 {
            break;
        }
        value /= 1024.0;
        unit = next_unit;
    }

    if unit == "B" {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {unit}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_human_readable_sizes() {
        assert_eq!(format_bytes(999), "999 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MiB");
    }

    #[test]
    fn reports_missing_when_directory_is_absent() {
        let statuses = collect_statuses(Path::new("/tmp/xrat-no-such-mmdb-dir-status-test"))
            .expect("status collection should succeed");

        assert_eq!(statuses.len(), 3);
        assert!(statuses.iter().all(EditionStatus::is_missing));
    }

    #[test]
    fn reports_present_size_when_file_exists() {
        let root = std::env::temp_dir().join("xrat-mmdb-status-present-test");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("root should be created");
        std::fs::write(root.join("GeoLite2-Country.mmdb"), [0_u8; 5]).expect("file should exist");

        let statuses = collect_statuses(&root).expect("status collection should succeed");
        let country = statuses
            .iter()
            .find(|status| status.file_name == "GeoLite2-Country.mmdb")
            .expect("country status should exist");

        assert_eq!(country.state, "present");
        assert_eq!(country.size_bytes, Some(5));
    }
}
