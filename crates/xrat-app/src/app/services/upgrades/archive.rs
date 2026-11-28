use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::app::{AppError, Result};

const MAX_BINARY_BYTES: u64 = 256 * 1024 * 1024;

pub(super) fn verify_checksum(archive: &Path, checksums: &str, filename: &str) -> Result<()> {
    let mut matches = checksums.lines().filter_map(|line| {
        let (hash, name) = line.split_once(char::is_whitespace)?;
        (name.trim().trim_start_matches('*') == filename).then_some(hash)
    });
    let expected = matches
        .next()
        .ok_or_else(|| AppError::InvalidArgument(format!("no checksum entry for {filename}")))?;
    if matches.next().is_some()
        || expected.len() != 64
        || !expected.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(AppError::InvalidArgument(format!(
            "invalid or duplicate checksum entry for {filename}"
        )));
    }
    let mut hasher = Sha256::new();
    std::io::copy(&mut File::open(archive)?, &mut hasher)?;
    if !format!("{:x}", hasher.finalize()).eq_ignore_ascii_case(expected) {
        return Err(AppError::InvalidArgument(format!(
            "checksum verification failed for {filename}"
        )));
    }
    Ok(())
}

pub(super) fn extract_binary(archive: &Path, directory: &Path, executable: &str) -> Result<()> {
    let mut staged = super::staging::StagedFile::new(directory)?;
    let mut found = false;
    if archive
        .extension()
        .is_some_and(|extension| extension == "zip")
    {
        let mut zip = zip::ZipArchive::new(File::open(archive)?)
            .map_err(|error| AppError::InvalidArgument(format!("invalid release ZIP: {error}")))?;
        for index in 0..zip.len() {
            let mut entry = zip
                .by_index(index)
                .map_err(|error| AppError::InvalidArgument(error.to_string()))?;
            let name = entry
                .name()
                .map_err(|error| AppError::InvalidArgument(error.to_string()))?;
            if name != executable && name != format!("./{executable}") {
                continue;
            }
            let mode = entry.unix_mode().unwrap_or(0) & 0o170000;
            if !entry.is_file() || !matches!(mode, 0 | 0o100000) {
                return Err(AppError::InvalidArgument(
                    "release executable must be a regular file".into(),
                ));
            }
            copy_entry(&mut entry, staged.as_file_mut(), &mut found)?;
        }
    } else {
        let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(File::open(archive)?));
        for entry in tar.entries()? {
            let mut entry = entry?;
            if entry.path()?.as_ref() != Path::new(executable)
                && entry.path()?.as_ref() != Path::new(&format!("./{executable}"))
            {
                continue;
            }
            if !entry.header().entry_type().is_file() {
                return Err(AppError::InvalidArgument(
                    "release executable must be a regular file".into(),
                ));
            }
            copy_entry(&mut entry, staged.as_file_mut(), &mut found)?;
        }
    }
    if !found {
        return Err(AppError::InvalidArgument(format!(
            "release archive is missing {executable}"
        )));
    }
    staged.as_file_mut().flush()?;
    staged.as_file().sync_all()?;
    staged.persist(directory.join(executable))?;
    Ok(())
}

fn copy_entry(reader: impl Read, target: &mut File, found: &mut bool) -> Result<()> {
    if *found {
        return Err(AppError::InvalidArgument(
            "duplicate executable in release archive".into(),
        ));
    }
    let size = std::io::copy(&mut reader.take(MAX_BINARY_BYTES + 1), target)?;
    if size == 0 || size > MAX_BINARY_BYTES {
        return Err(AppError::InvalidArgument(
            "release executable has invalid size".into(),
        ));
    }
    *found = true;
    Ok(())
}

#[cfg(test)]
#[path = "archive_tests.rs"]
mod tests;
