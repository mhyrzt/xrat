use std::io::IsTerminal;

use indicatif::MultiProgress;

use super::super::edition::MmdbEdition;
use crate::app::commands::progress::{CliProgress, should_enable};

pub(crate) struct DownloadProgressSet {
    multi: Option<MultiProgress>,
}

impl DownloadProgressSet {
    pub(crate) fn new(quiet: bool) -> Self {
        let enabled = should_enable(!quiet, std::io::stderr().is_terminal());
        Self {
            multi: enabled.then(MultiProgress::new),
        }
    }

    pub(crate) fn create_bar(&self, edition: MmdbEdition) -> CliProgress {
        match &self.multi {
            Some(multi) => CliProgress::bytes_bar_in_multi(true, multi, None, edition.to_string()),
            None => CliProgress::disabled(),
        }
    }
}

pub(crate) fn create_progress_bar(
    edition: MmdbEdition,
    quiet: bool,
    content_length: Option<u64>,
) -> CliProgress {
    CliProgress::bytes_bar(!quiet, content_length, edition.to_string())
}

pub(crate) fn ensure_non_empty_download(
    bytes_written: u64,
    progress: &CliProgress,
    edition: MmdbEdition,
    url: &str,
) -> crate::app::Result<()> {
    use crate::app::AppError;

    if bytes_written != 0 {
        return Ok(());
    }

    progress.abandon_with_message("failed: empty file");

    Err(AppError::GeoipDownload {
        edition: edition.to_string(),
        url: url.to_string(),
        reason: "download failed or returned empty file".to_string(),
    })
}
