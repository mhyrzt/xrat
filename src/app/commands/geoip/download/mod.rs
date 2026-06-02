mod executor;
mod progress;
mod request;
mod summary;

#[cfg(test)]
mod tests;

use crate::app::AppError;
use crate::app::context::AppContext;
use crate::cli::GeoIpDownloadArgs;

use executor::execute_downloads;
use request::DownloadRequest;

pub(crate) async fn run(context: &AppContext, args: &GeoIpDownloadArgs) -> crate::app::Result<()> {
    let request = DownloadRequest::from_cli(context, args)?;
    let summary = execute_downloads(&request).await;
    summary.print();

    if !summary.failed.is_empty() {
        return Err(AppError::InvalidArgument(format!(
            "one or more GeoIP downloads failed: {}",
            summary.format_failure_details()
        )));
    }

    Ok(())
}
