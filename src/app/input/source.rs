use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::db::{ImportSource, SourceKind};
use crate::support::url::looks_like_url;

pub fn read_input(input: &str) -> crate::app::Result<(ImportSource, Vec<u8>)> {
    if looks_like_url(input) {
        return Ok((
            ImportSource {
                kind: SourceKind::Url,
                value: input.to_string(),
                name: None,
            },
            fetch_url(input)?,
        ));
    }

    read_non_url_input(input)
}

pub async fn read_input_async(input: &str) -> crate::app::Result<(ImportSource, Vec<u8>)> {
    if looks_like_url(input) {
        return Ok((
            ImportSource {
                kind: SourceKind::Url,
                value: input.to_string(),
                name: None,
            },
            fetch_url_async(input).await?,
        ));
    }

    read_non_url_input(input)
}

pub fn fetch_url(url: &str) -> crate::app::Result<Vec<u8>> {
    let response = reqwest::blocking::get(url)?.error_for_status()?;
    Ok(response.bytes()?.to_vec())
}

pub async fn fetch_url_async(url: &str) -> crate::app::Result<Vec<u8>> {
    let response = reqwest::get(url).await?.error_for_status()?;
    Ok(response.bytes().await?.to_vec())
}

fn read_non_url_input(input: &str) -> crate::app::Result<(ImportSource, Vec<u8>)> {
    let path = Path::new(input);
    if path.exists() {
        return Ok((
            ImportSource {
                kind: SourceKind::File,
                value: input.to_string(),
                name: path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned()),
            },
            fs::read(path)?,
        ));
    }

    Ok((
        ImportSource {
            kind: SourceKind::RawText,
            value: input.to_string(),
            name: None,
        },
        input.as_bytes().to_vec(),
    ))
}

pub fn save_json<T: Serialize>(output_path: &Path, value: &T) -> crate::app::Result<()> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let body = serde_json::to_string_pretty(value)?;
    fs::write(output_path, body)?;
    Ok(())
}
