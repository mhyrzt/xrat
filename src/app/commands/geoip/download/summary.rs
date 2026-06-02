use super::super::edition::MmdbEdition;

#[derive(Debug, Default)]
pub(crate) struct DownloadSummary {
    pub(crate) downloaded: usize,
    pub(crate) skipped: usize,
    pub(crate) failed: Vec<DownloadFailure>,
}

#[derive(Debug)]
pub(crate) struct DownloadFailure {
    pub(crate) edition: MmdbEdition,
    pub(crate) reason: String,
}

impl DownloadSummary {
    pub(crate) fn print(&self) {
        println!("{}", self.format());
    }

    pub(crate) fn format(&self) -> String {
        format!(
            "summary: downloaded={} skipped={} failed={}",
            self.downloaded,
            self.skipped,
            self.failed.len()
        )
    }

    pub(crate) fn format_failure_details(&self) -> String {
        self.failed
            .iter()
            .map(|failure| format!("{} ({})", failure.edition, failure.reason))
            .collect::<Vec<_>>()
            .join(", ")
    }
}
