use std::fs;
use std::io::Write;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use tempfile::NamedTempFile;

use crate::app::AppError;
use crate::app::context::AppContext;
use crate::cli::GeoIpDownloadArgs;

use super::edition::{MmdbEdition, SUPPORTED_EDITIONS};
use super::{ensure_mmdb_target_dir, mmdb_file_path, resolve_mmdb_target_dir};

pub(crate) async fn run(context: &AppContext, args: &GeoIpDownloadArgs) -> crate::app::Result<()> {
    let request = DownloadRequest::from_cli(context, args)?;
    let summary = execute_downloads(&request).await;
    print_summary(&summary);

    if !summary.failed.is_empty() {
        return Err(AppError::InvalidArgument(format!(
            "one or more GeoIP downloads failed: {}",
            summary
                .failed
                .iter()
                .map(|failure| failure.edition.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub(crate) struct DownloadRequest {
    pub(crate) editions: Vec<MmdbEdition>,
    pub(crate) mmdb_dir: std::path::PathBuf,
    pub(crate) force: bool,
    pub(crate) url_template: String,
    pub(crate) timeout_secs: u64,
    pub(crate) quiet: bool,
}

impl DownloadRequest {
    fn from_cli(context: &AppContext, args: &GeoIpDownloadArgs) -> crate::app::Result<Self> {
        Ok(Self {
            editions: resolve_requested_editions(&args.editions, args.all)?,
            mmdb_dir: resolve_mmdb_target_dir(context, args.output.as_ref()),
            force: args.force,
            url_template: args
                .url
                .clone()
                .unwrap_or_else(|| context.app_config.mmdb.download_url.clone()),
            timeout_secs: args
                .timeout_secs
                .unwrap_or(context.app_config.mmdb.timeout_secs),
            quiet: args.quiet,
        })
    }
}

#[derive(Debug, Default)]
struct DownloadSummary {
    downloaded: usize,
    skipped: usize,
    failed: Vec<DownloadFailure>,
}

#[derive(Debug)]
struct DownloadFailure {
    edition: MmdbEdition,
}

pub(crate) async fn execute_downloads(request: &DownloadRequest) -> DownloadSummary {
    let mut summary = DownloadSummary::default();

    if let Err(error) = ensure_mmdb_target_dir(&request.mmdb_dir) {
        eprintln!("error: {error}");
        summary.failed = request
            .editions
            .iter()
            .copied()
            .map(|edition| DownloadFailure { edition })
            .collect();
        return summary;
    }

    for edition in &request.editions {
        match download_one(request, *edition).await {
            Ok(DownloadOutcome::Downloaded) => summary.downloaded += 1,
            Ok(DownloadOutcome::Skipped) => summary.skipped += 1,
            Err(error) => {
                eprintln!("error: {error}");
                summary.failed.push(DownloadFailure { edition: *edition });
            }
        }
    }

    summary
}

enum DownloadOutcome {
    Downloaded,
    Skipped,
}

async fn download_one(
    request: &DownloadRequest,
    edition: MmdbEdition,
) -> crate::app::Result<DownloadOutcome> {
    let destination = mmdb_file_path(&request.mmdb_dir, edition);
    if destination.exists() && !request.force {
        println!(
            "skipped: {} (already present, use --force to redownload)",
            edition.file_name()
        );
        return Ok(DownloadOutcome::Skipped);
    }

    let url = build_download_url(&request.url_template, edition);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(request.timeout_secs))
        .build()?;
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|error| AppError::GeoipDownload {
            edition: edition.to_string(),
            url: url.clone(),
            reason: error.to_string(),
        })?;

    if !response.status().is_success() {
        return Err(AppError::GeoipDownload {
            edition: edition.to_string(),
            url,
            reason: format!("HTTP {}", response.status()),
        });
    }

    let progress = create_progress_bar(edition, request.quiet, response.content_length());
    let mut file = NamedTempFile::new_in(&request.mmdb_dir)?;
    let mut bytes_written = 0_u64;
    let mut response = response;

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| AppError::GeoipDownload {
            edition: edition.to_string(),
            url: url.clone(),
            reason: error.to_string(),
        })?
    {
        file.write_all(&chunk)?;
        bytes_written += chunk.len() as u64;
        if let Some(progress) = &progress {
            progress.inc(chunk.len() as u64);
        }
    }

    ensure_non_empty_download(bytes_written, &progress, edition, &url)?;

    file.flush()?;
    let _persisted = file.persist(&destination).map_err(|error| error.error)?;
    set_mmdb_permissions(&destination)?;

    if let Some(progress) = progress {
        progress.finish_with_message("done".to_string());
    }
    println!(
        "downloaded: {} -> {}",
        edition.file_name(),
        destination.display()
    );

    Ok(DownloadOutcome::Downloaded)
}

fn set_mmdb_permissions(path: &std::path::Path) -> crate::app::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o644))?;
    }

    #[cfg(not(unix))]
    {
        let _ = path;
    }

    Ok(())
}

fn create_progress_bar(
    edition: MmdbEdition,
    quiet: bool,
    content_length: Option<u64>,
) -> Option<ProgressBar> {
    use std::io::IsTerminal;

    if quiet || !std::io::stderr().is_terminal() {
        return None;
    }

    let progress = ProgressBar::new(content_length.unwrap_or(0));
    let style = ProgressStyle::with_template(
        "{spinner:.green} {msg} [{bar:32.cyan/blue}] {bytes}/{total_bytes}",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar())
    .progress_chars("=>-");
    progress.set_style(style);
    progress.set_message(edition.to_string());

    Some(progress)
}

fn print_summary(summary: &DownloadSummary) {
    println!("{}", format_summary(summary));
}

fn format_summary(summary: &DownloadSummary) -> String {
    format!(
        "summary: downloaded={} skipped={} failed={}",
        summary.downloaded,
        summary.skipped,
        summary.failed.len()
    )
}

fn ensure_non_empty_download(
    bytes_written: u64,
    progress: &Option<ProgressBar>,
    edition: MmdbEdition,
    url: &str,
) -> crate::app::Result<()> {
    if bytes_written != 0 {
        return Ok(());
    }

    if let Some(progress) = progress {
        progress.abandon_with_message("failed: empty file".to_string());
    }

    Err(AppError::GeoipDownload {
        edition: edition.to_string(),
        url: url.to_string(),
        reason: "download failed or returned empty file".to_string(),
    })
}

pub(crate) fn build_download_url(template: &str, edition: MmdbEdition) -> String {
    template.replace("{edition}", edition.canonical_name())
}

pub(crate) fn resolve_requested_editions(
    values: &[String],
    all: bool,
) -> crate::app::Result<Vec<MmdbEdition>> {
    if all {
        return Ok(SUPPORTED_EDITIONS.to_vec());
    }

    if values.is_empty() {
        return Ok(vec![MmdbEdition::Country]);
    }

    let mut resolved = Vec::with_capacity(values.len());
    for value in values {
        let edition: MmdbEdition = value.parse()?;
        if !resolved.contains(&edition) {
            resolved.push(edition);
        }
    }

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_request(mmdb_dir: std::path::PathBuf) -> DownloadRequest {
        DownloadRequest {
            editions: vec![MmdbEdition::Country],
            mmdb_dir,
            force: false,
            url_template: "https://example.com/{edition}.mmdb".to_string(),
            timeout_secs: 1,
            quiet: true,
        }
    }

    #[test]
    fn builds_download_url_from_template() {
        assert_eq!(
            build_download_url("https://example.com/{edition}.mmdb", MmdbEdition::Asn),
            "https://example.com/GeoLite2-ASN.mmdb"
        );
    }

    #[test]
    fn defaults_download_to_country() {
        assert_eq!(
            resolve_requested_editions(&[], false).unwrap(),
            vec![MmdbEdition::Country]
        );
    }

    #[test]
    fn resolves_all_download_editions() {
        assert_eq!(
            resolve_requested_editions(&[], true).unwrap(),
            SUPPORTED_EDITIONS.to_vec()
        );
    }

    #[test]
    fn formats_summary_line() {
        let summary = DownloadSummary {
            downloaded: 2,
            skipped: 1,
            failed: vec![DownloadFailure {
                edition: MmdbEdition::Asn,
            }],
        };

        assert_eq!(
            format_summary(&summary),
            "summary: downloaded=2 skipped=1 failed=1"
        );
    }

    #[test]
    fn rejects_empty_downloads() {
        let error = ensure_non_empty_download(
            0,
            &None,
            MmdbEdition::City,
            "https://example.com/GeoLite2-City.mmdb",
        )
        .expect_err("empty download should fail");

        match error {
            AppError::GeoipDownload {
                edition,
                url,
                reason,
            } => {
                assert_eq!(edition, "GeoLite2-City");
                assert_eq!(url, "https://example.com/GeoLite2-City.mmdb");
                assert!(reason.contains("empty file"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn skips_existing_destination_without_force() {
        let root = std::env::temp_dir().join("xrat-geoip-download-skip-test");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("root should be created");
        std::fs::write(root.join("GeoLite2-Country.mmdb"), [1_u8; 4])
            .expect("fixture file should exist");

        let outcome = download_one(&test_request(root), MmdbEdition::Country)
            .await
            .expect("existing file should skip");

        assert!(matches!(outcome, DownloadOutcome::Skipped));
    }
}
