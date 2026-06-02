use std::fs;
use std::io::Write;
use std::time::Duration;

use tempfile::NamedTempFile;

use crate::app::AppError;

use super::super::edition::MmdbEdition;
use super::super::{ensure_mmdb_target_dir, mmdb_file_path};
use super::progress::{create_progress_bar, ensure_non_empty_download};
use super::request::DownloadRequest;
use super::summary::{DownloadFailure, DownloadSummary};

pub(crate) async fn execute_downloads(request: &DownloadRequest) -> DownloadSummary {
    let mut summary = DownloadSummary::default();

    if let Err(error) = ensure_mmdb_target_dir(&request.mmdb_dir) {
        eprintln!("error: {error}");
        summary.failed = request
            .editions
            .iter()
            .copied()
            .map(|edition| DownloadFailure {
                edition,
                reason: error.to_string(),
            })
            .collect();
        return summary;
    }

    let request = std::sync::Arc::new(request.clone());
    let mut join_set = tokio::task::JoinSet::new();

    for edition in &request.editions {
        let request = std::sync::Arc::clone(&request);
        let edition = *edition;
        join_set.spawn(async move {
            let result = download_one(&request, edition).await;
            (edition, result)
        });
    }

    while let Some(join_result) = join_set.join_next().await {
        match join_result {
            Ok((_, Ok(DownloadOutcome::Downloaded))) => summary.downloaded += 1,
            Ok((_, Ok(DownloadOutcome::Skipped))) => summary.skipped += 1,
            Ok((edition, Err(error))) => {
                eprintln!("error: {error}");
                summary.failed.push(DownloadFailure {
                    edition,
                    reason: error.to_string(),
                });
            }
            Err(join_error) => {
                eprintln!("error: task failed: {join_error}");
                summary.failed.push(DownloadFailure {
                    edition: MmdbEdition::Country,
                    reason: format!("task failed: {join_error}"),
                });
            }
        }
    }

    summary
}

#[derive(Debug)]
pub(crate) enum DownloadOutcome {
    Downloaded,
    Skipped,
}

pub(crate) async fn download_one(
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

    let _url = build_download_url(&request.url_template, edition);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(request.timeout_secs))
        .build()?;
    download_one_with_client(&client, request, edition).await
}

pub(crate) async fn download_one_with_client(
    client: &reqwest::Client,
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

pub(crate) fn build_download_url(template: &str, edition: MmdbEdition) -> String {
    template.replace("{edition}", edition.canonical_name())
}
