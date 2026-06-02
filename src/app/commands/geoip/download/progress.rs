use indicatif::{ProgressBar, ProgressStyle};

use super::super::edition::MmdbEdition;

pub(crate) fn create_progress_bar(
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

pub(crate) fn ensure_non_empty_download(
    bytes_written: u64,
    progress: &Option<ProgressBar>,
    edition: MmdbEdition,
    url: &str,
) -> crate::app::Result<()> {
    use crate::app::AppError;

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
