use std::path::PathBuf;

use crate::app::context::AppContext;
use crate::cli::GeoIpDownloadArgs;

use super::super::edition::{MmdbEdition, SUPPORTED_EDITIONS};
use super::super::resolve_mmdb_target_dir;

#[derive(Clone, Debug)]
pub(crate) struct DownloadRequest {
    pub(crate) editions: Vec<MmdbEdition>,
    pub(crate) mmdb_dir: PathBuf,
    pub(crate) force: bool,
    pub(crate) url_template: String,
    pub(crate) timeout_secs: u64,
    pub(crate) quiet: bool,
}

impl DownloadRequest {
    pub(crate) fn from_cli(
        context: &AppContext,
        args: &GeoIpDownloadArgs,
    ) -> crate::app::Result<Self> {
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
