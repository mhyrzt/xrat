use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};

pub(crate) const ARCHIVE_URL: &str = "https://www.wintun.net/builds/wintun-0.14.1.zip";
const ARCHIVE_HASH: &str = "07c256185d6ee3652e09fa55c0b673e2624b565e02c4b9091c79ca7d2f24ef51";
const LICENSE_HASH: &str = "183adac21e7d96c508c8fd34d394b7b6708bc81564ad1bad61ab66143a008cd2";
const MEMBER_LIMIT: u64 = 1024 * 1024;
pub(crate) const ARCHIVE_LIMIT: usize = 4 * 1024 * 1024;

pub(crate) struct Package {
    pub dll: Vec<u8>,
    pub license: Vec<u8>,
}

pub(crate) fn prepare(bytes: &[u8], arch: &str) -> Result<Package, String> {
    let (directory, dll_hash) = architecture(arch)?;
    if bytes.len() > ARCHIVE_LIMIT || digest(bytes) != ARCHIVE_HASH {
        return Err("Wintun archive does not match the pinned official 0.14.1 SHA-256".into());
    }
    extract(bytes, directory, dll_hash, LICENSE_HASH)
}

fn architecture(arch: &str) -> Result<(&'static str, &'static str), String> {
    match arch {
        "x86_64" => Ok((
            "amd64",
            "e5da8447dc2c320edc0fc52fa01885c103de8c118481f683643cacc3220dafce",
        )),
        "aarch64" => Ok((
            "arm64",
            "f7ba89005544be9d85231a9e0d5f23b2d15b3311667e2dad0debd344918a3f80",
        )),
        _ => Err("managed Windows TUN supports only x64 and ARM64".into()),
    }
}

fn extract(
    bytes: &[u8],
    directory: &str,
    dll_hash: &str,
    license_hash: &str,
) -> Result<Package, String> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|error| error.to_string())?;
    let dll = member(
        &mut archive,
        &format!("wintun/bin/{directory}/wintun.dll"),
        dll_hash,
    )?;
    let license = member(&mut archive, "wintun/LICENSE.txt", license_hash)?;
    Ok(Package { dll, license })
}

fn member(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    wanted: &str,
    hash: &str,
) -> Result<Vec<u8>, String> {
    let mut found = None;
    for index in 0..archive.len() {
        let entry = archive.by_index(index).map_err(|error| error.to_string())?;
        if entry.name().map_err(|error| error.to_string())? != wanted {
            continue;
        }
        if found.replace(index).is_some() {
            return Err(format!("Wintun archive contains duplicate {wanted}"));
        }
        if !entry.is_file()
            || entry.is_symlink()
            || entry.size() == 0
            || entry.size() > MEMBER_LIMIT
        {
            return Err(format!("invalid Wintun archive member {wanted}"));
        }
    }
    let index = found.ok_or_else(|| format!("Wintun archive is missing {wanted}"))?;
    let mut output = Vec::new();
    archive
        .by_index(index)
        .map_err(|error| error.to_string())?
        .take(MEMBER_LIMIT + 1)
        .read_to_end(&mut output)
        .map_err(|error| error.to_string())?;
    if output.is_empty() || output.len() as u64 > MEMBER_LIMIT || digest(&output) != hash {
        return Err(format!(
            "Wintun member {wanted} failed its pinned SHA-256 verification"
        ));
    }
    Ok(output)
}

#[cfg(windows)]
pub(crate) fn verify_installed(directory: &std::path::Path) -> Result<(), String> {
    let (_, hash) = architecture(std::env::consts::ARCH)?;
    for (name, expected) in [("wintun.dll", hash), ("wintun-LICENSE.txt", LICENSE_HASH)] {
        let mut bytes = Vec::new();
        std::fs::File::open(directory.join(name))
            .map_err(|error| format!("{name}: {error}"))?
            .take(MEMBER_LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        if bytes.len() as u64 > MEMBER_LIMIT || digest(&bytes) != expected {
            return Err(format!(
                "{name} differs from managed Wintun 0.14.1; rerun elevated machine tun setup"
            ));
        }
    }
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
pub(super) fn prepare_fixture(
    bytes: &[u8],
    directory: &str,
    dll: &[u8],
    license: &[u8],
) -> Result<Package, String> {
    extract(bytes, directory, &digest(dll), &digest(license))
}
