use std::fs;
use std::path::Path;

use serde::Serialize;

pub fn read_input(input: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if looks_like_url(input) {
        return fetch_url(input);
    }

    Ok(fs::read(input)?)
}

pub fn fetch_url(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?.error_for_status()?;
    Ok(response.bytes()?.to_vec())
}

pub fn save_json<T: Serialize>(
    output_path: &Path,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let body = serde_json::to_string_pretty(value)?;
    fs::write(output_path, body)?;
    Ok(())
}

fn looks_like_url(input: &str) -> bool {
    matches!(
        input.split_once("://").map(|(scheme, _)| scheme),
        Some("http" | "https")
    )
}
