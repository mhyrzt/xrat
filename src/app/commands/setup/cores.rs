use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use flate2::read::GzDecoder;
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::app::commands::progress::CliProgress;
use crate::app::config;
use crate::app::context::AppContext;
use crate::support::platform;

pub(super) const CORE_KINDS: [CoreKind; 3] = [CoreKind::Xray, CoreKind::SingBox, CoreKind::V2Ray];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CoreKind {
    Xray,
    SingBox,
    V2Ray,
}

impl CoreKind {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Xray => "xray",
            Self::SingBox => "sing-box",
            Self::V2Ray => "v2ray",
        }
    }

    fn config_key(self) -> &'static str {
        match self {
            Self::Xray => "xray",
            Self::SingBox => "sing_box",
            Self::V2Ray => "v2ray",
        }
    }

    pub(super) fn repository(self) -> &'static str {
        match self {
            Self::Xray => "XTLS/Xray-core",
            Self::SingBox => "SagerNet/sing-box",
            Self::V2Ray => "v2fly/v2ray-core",
        }
    }

    pub(super) fn required(self) -> bool {
        self == Self::Xray
    }

    pub(super) fn unattended_default(self) -> bool {
        matches!(self, Self::Xray | Self::SingBox)
    }
}

#[derive(Clone, Debug)]
pub(super) struct CoreRelease {
    pub(super) version: Version,
    tag: String,
    asset: ReleaseAsset,
}

#[derive(Clone, Debug)]
struct ReleaseAsset {
    name: String,
    url: String,
    sha256: String,
}

#[derive(Debug)]
pub(super) struct CoreProbe {
    pub(super) kind: CoreKind,
    pub(super) path: Option<PathBuf>,
    pub(super) version: Option<Version>,
    pub(super) managed: bool,
    pub(super) latest: Result<CoreRelease, String>,
}

impl CoreProbe {
    pub(super) fn missing(&self) -> bool {
        self.path.is_none()
    }

    pub(super) fn outdated(&self) -> bool {
        matches!((&self.version, &self.latest), (Some(current), Ok(latest)) if current < &latest.version)
    }

    pub(super) fn detail(&self) -> String {
        let location = self
            .path
            .as_deref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "not installed".to_string());
        let ownership = if self.managed { "managed" } else { "external" };
        let current = self
            .version
            .as_ref()
            .map(|version| format!("v{version}"))
            .unwrap_or_else(|| "version unknown".to_string());
        match &self.latest {
            Ok(latest) if self.path.is_some() => {
                format!(
                    "{location} ({current}; latest v{}; {ownership})",
                    latest.version
                )
            }
            Ok(latest) => format!("{location} (latest v{})", latest.version),
            Err(error) if self.path.is_some() => {
                format!("{location} ({current}; {ownership}; update check failed: {error})")
            }
            Err(error) => format!("{location} (update check failed: {error})"),
        }
    }
}

#[derive(Debug)]
pub(super) struct InstallResult {
    pub(super) binary_path: PathBuf,
    pub(super) version: Version,
    pub(super) cli_link_warning: Option<String>,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    prerelease: bool,
    created_at: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    digest: Option<String>,
}

pub(super) async fn probe_all(context: &AppContext) -> Vec<CoreProbe> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(concat!("xrat/", env!("CARGO_PKG_VERSION")))
        .build();

    let (xray, sing_box, v2ray) = match client {
        Ok(client) => tokio::join!(
            fetch_latest(&client, CoreKind::Xray),
            fetch_latest(&client, CoreKind::SingBox),
            fetch_latest(&client, CoreKind::V2Ray),
        ),
        Err(error) => {
            let message = error.to_string();
            (Err(message.clone()), Err(message.clone()), Err(message))
        }
    };
    let latest = [xray, sing_box, v2ray];
    let managed_root = managed_root().ok();

    CORE_KINDS
        .into_iter()
        .zip(latest)
        .map(|(kind, latest)| {
            let path = resolve_installed_path(configured_path(context, kind));
            let version = path.as_deref().and_then(binary_version);
            let managed = path
                .as_deref()
                .zip(managed_root.as_deref())
                .is_some_and(|(path, root)| path.starts_with(root));
            CoreProbe {
                kind,
                path,
                version,
                managed,
                latest,
            }
        })
        .collect()
}

fn configured_path(context: &AppContext, kind: CoreKind) -> &Path {
    match kind {
        CoreKind::Xray => &context.runtime_paths.xray_path,
        CoreKind::SingBox => &context.runtime_paths.sing_box_path,
        CoreKind::V2Ray => &context.runtime_paths.v2ray_path,
    }
}

fn resolve_installed_path(configured: &Path) -> Option<PathBuf> {
    if configured.is_file() {
        return Some(configured.to_path_buf());
    }
    if configured.components().count() == 1 {
        return configured.to_str().and_then(platform::binary_on_path);
    }
    None
}

async fn fetch_latest(client: &reqwest::Client, kind: CoreKind) -> Result<CoreRelease, String> {
    fetch_release(client, kind, None, false).await
}

pub(super) async fn fetch_release(
    client: &reqwest::Client,
    kind: CoreKind,
    version: Option<&Version>,
    prerelease: bool,
) -> Result<CoreRelease, String> {
    let selector = if prerelease {
        "latest prerelease".to_string()
    } else if let Some(version) = version {
        format!("v{version}")
    } else {
        "latest stable".to_string()
    };
    tracing::info!(core = kind.name(), %selector, "resolving proxy core release");
    let response = client
        .get(release_api_url(kind, version, prerelease))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("GitHub returned HTTP {}", response.status()));
    }
    let body = response.text().await.map_err(|error| error.to_string())?;
    let payload = if prerelease {
        let releases: Vec<GithubRelease> =
            serde_json::from_str(&body).map_err(|error| error.to_string())?;
        newest_prerelease(releases)
            .ok_or_else(|| format!("{} has no published prerelease", kind.repository()))?
    } else {
        serde_json::from_str(&body).map_err(|error| error.to_string())?
    };
    let release = release_from_payload(kind, payload)?;
    if let Some(version) = version
        && release.version != *version
    {
        return Err(format!(
            "GitHub returned v{} when v{version} was requested",
            release.version
        ));
    }
    tracing::info!(
        core = kind.name(),
        version = %release.version,
        tag = %release.tag,
        asset = %release.asset.name,
        %selector,
        "proxy core release resolved"
    );
    Ok(release)
}

fn release_api_url(kind: CoreKind, version: Option<&Version>, prerelease: bool) -> String {
    if prerelease {
        return format!(
            "https://api.github.com/repos/{}/releases?per_page=100",
            kind.repository()
        );
    }
    let release = version
        .map(|version| format!("tags/v{version}"))
        .unwrap_or_else(|| "latest".to_string());
    format!(
        "https://api.github.com/repos/{}/releases/{release}",
        kind.repository()
    )
}

fn newest_prerelease(releases: Vec<GithubRelease>) -> Option<GithubRelease> {
    releases
        .into_iter()
        .filter(|release| release.prerelease)
        .max_by(|left, right| left.created_at.cmp(&right.created_at))
}

fn release_from_payload(kind: CoreKind, payload: GithubRelease) -> Result<CoreRelease, String> {
    let version = Version::parse(payload.tag_name.trim_start_matches('v'))
        .map_err(|error| format!("invalid release tag {}: {error}", payload.tag_name))?;
    let expected_name = asset_name(kind, &version)?;
    let asset = payload
        .assets
        .into_iter()
        .find(|asset| asset.name == expected_name)
        .ok_or_else(|| format!("release {} has no {expected_name}", payload.tag_name))?;
    let sha256 = parse_sha256(asset.digest.as_deref())?;
    Ok(CoreRelease {
        version,
        tag: payload.tag_name,
        asset: ReleaseAsset {
            name: asset.name,
            url: asset.browser_download_url,
            sha256,
        },
    })
}

fn asset_name(kind: CoreKind, version: &Version) -> Result<String, String> {
    asset_name_for(kind, version, platform::os(), platform::arch())
}

fn asset_name_for(
    kind: CoreKind,
    version: &Version,
    os: &str,
    arch: &str,
) -> Result<String, String> {
    match kind {
        CoreKind::Xray | CoreKind::V2Ray => {
            let prefix = if kind == CoreKind::Xray {
                "Xray"
            } else {
                "v2ray"
            };
            let os = match os {
                "linux" => "linux",
                "macos" => "macos",
                _ => {
                    return Err(format!(
                        "managed installation is unsupported on {os}/{arch}"
                    ));
                }
            };
            let arch = match arch {
                "x86_64" => "64",
                "aarch64" => "arm64-v8a",
                _ => {
                    return Err(format!(
                        "managed installation is unsupported on {os}/{arch}"
                    ));
                }
            };
            Ok(format!("{prefix}-{os}-{arch}.zip"))
        }
        CoreKind::SingBox => {
            let os = match os {
                "linux" => "linux",
                "macos" => "darwin",
                _ => {
                    return Err(format!(
                        "managed installation is unsupported on {os}/{arch}"
                    ));
                }
            };
            let arch = match arch {
                "x86_64" => "amd64",
                "aarch64" => "arm64",
                _ => {
                    return Err(format!(
                        "managed installation is unsupported on {os}/{arch}"
                    ));
                }
            };
            Ok(format!("sing-box-{version}-{os}-{arch}.tar.gz"))
        }
    }
}

fn parse_sha256(digest: Option<&str>) -> Result<String, String> {
    let digest = digest.ok_or_else(|| "release asset has no published digest".to_string())?;
    let value = digest
        .strip_prefix("sha256:")
        .ok_or_else(|| format!("unsupported release digest {digest:?}"))?;
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("release asset has an invalid SHA-256 digest".to_string());
    }
    Ok(value.to_ascii_lowercase())
}

pub(super) async fn install(
    context: &AppContext,
    kind: CoreKind,
    release: &CoreRelease,
    progress_enabled: bool,
) -> Result<InstallResult, String> {
    tracing::info!(
        core = kind.name(),
        version = %release.version,
        asset = %release.asset.name,
        "proxy core installation started"
    );
    let result = install_inner(context, kind, release, progress_enabled).await;
    if let Err(error) = &result {
        tracing::error!(
            core = kind.name(),
            version = %release.version,
            error = %error,
            "proxy core installation failed"
        );
    }
    result
}

async fn install_inner(
    context: &AppContext,
    kind: CoreKind,
    release: &CoreRelease,
    progress_enabled: bool,
) -> Result<InstallResult, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .user_agent(concat!("xrat/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| error.to_string())?;
    let bytes = download_archive(&client, kind, release, progress_enabled).await?;
    install_archive(context, kind, release, &bytes)
}

async fn download_archive(
    client: &reqwest::Client,
    kind: CoreKind,
    release: &CoreRelease,
    progress_enabled: bool,
) -> Result<Vec<u8>, String> {
    tracing::info!(
        core = kind.name(),
        version = %release.version,
        asset = %release.asset.name,
        "proxy core download started"
    );
    let connection_progress = CliProgress::spinner(
        progress_enabled,
        format!("starting {} v{} download", kind.name(), release.version),
    );
    let mut response = match client.get(&release.asset.url).send().await {
        Ok(response) => {
            connection_progress.finish_and_clear();
            response
        }
        Err(error) => {
            connection_progress.abandon_with_message(format!("{} download failed", kind.name()));
            return Err(format!(
                "could not download {}: {error}",
                release.asset.name
            ));
        }
    };
    if !response.status().is_success() {
        return Err(format!(
            "could not download {}: HTTP {}",
            release.asset.name,
            response.status()
        ));
    }
    let content_length = response.content_length();
    let progress = CliProgress::bytes_bar(
        progress_enabled,
        content_length,
        format!("downloading {} v{}", kind.name(), release.version),
    );
    let mut bytes = Vec::new();
    loop {
        match response.chunk().await {
            Ok(Some(chunk)) => {
                bytes.extend_from_slice(&chunk);
                progress.inc(chunk.len() as u64);
            }
            Ok(None) => break,
            Err(error) => {
                progress.abandon_with_message(format!("{} download failed", kind.name()));
                return Err(format!(
                    "could not download {}: {error}",
                    release.asset.name
                ));
            }
        }
    }
    progress.finish_with_message(format!("downloaded {} v{}", kind.name(), release.version));
    tracing::info!(
        core = kind.name(),
        version = %release.version,
        bytes = bytes.len(),
        content_length = ?content_length,
        "proxy core download completed"
    );
    Ok(bytes)
}

fn install_archive(
    context: &AppContext,
    kind: CoreKind,
    release: &CoreRelease,
    bytes: &[u8],
) -> Result<InstallResult, String> {
    tracing::info!(
        core = kind.name(),
        version = %release.version,
        "verifying proxy core checksum"
    );
    verify_checksum(&release.asset, bytes)?;
    tracing::info!(
        core = kind.name(),
        version = %release.version,
        "proxy core checksum verified"
    );

    let root = managed_root()?;
    fs::create_dir_all(&root)
        .map_err(|error| format!("could not create core directory: {error}"))?;
    let staging = tempfile::Builder::new()
        .prefix(".install-")
        .tempdir_in(&root)
        .map_err(|error| format!("could not create staging directory: {error}"))?;
    let payload = staging.path().join("payload");
    fs::create_dir(&payload).map_err(|error| error.to_string())?;
    tracing::info!(
        core = kind.name(),
        version = %release.version,
        "extracting proxy core archive"
    );
    extract_archive(kind, bytes, &payload)?;
    tracing::info!(
        core = kind.name(),
        version = %release.version,
        "proxy core archive extracted"
    );

    let staged_binary = payload.join(kind.name());
    set_executable(&staged_binary)?;
    let staged_version = binary_version(&staged_binary)
        .ok_or_else(|| format!("staged {} did not report a version", kind.name()))?;
    if staged_version != release.version {
        return Err(format!(
            "staged {} reported v{staged_version}, expected {}",
            kind.name(),
            release.tag
        ));
    }
    tracing::info!(
        core = kind.name(),
        version = %staged_version,
        "staged proxy core validated"
    );

    let target = root.join(kind.name());
    tracing::info!(
        core = kind.name(),
        version = %release.version,
        path = %target.display(),
        "activating managed proxy core"
    );
    replace_directory(&payload, &target)?;
    let binary_path = target.join(kind.name());
    let cli_link_warning = ensure_cli_link(kind, &binary_path, &root)?;
    config::update_runtime_binary_path(
        &context.runtime_paths.config_path,
        kind.config_key(),
        &binary_path,
    )
    .map_err(|error| format!("installed core but could not update config: {error}"))?;
    tracing::info!(
        core = kind.name(),
        version = %staged_version,
        path = %binary_path.display(),
        "managed proxy core activated"
    );

    Ok(InstallResult {
        binary_path,
        version: staged_version,
        cli_link_warning,
    })
}

fn verify_checksum(asset: &ReleaseAsset, bytes: &[u8]) -> Result<(), String> {
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual == asset.sha256 {
        return Ok(());
    }
    Err(format!(
        "SHA-256 mismatch for {} (expected {}, got {actual})",
        asset.name, asset.sha256
    ))
}

fn managed_root() -> Result<PathBuf, String> {
    platform::xdg_data_home()
        .map(|path| path.join("xrat").join("cores"))
        .ok_or_else(|| "could not determine the user data directory".to_string())
}

fn extract_archive(kind: CoreKind, bytes: &[u8], destination: &Path) -> Result<(), String> {
    match kind {
        CoreKind::Xray | CoreKind::V2Ray => extract_zip(kind, bytes, destination),
        CoreKind::SingBox => extract_sing_box(bytes, destination),
    }
}

fn extract_zip(kind: CoreKind, bytes: &[u8], destination: &Path) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("invalid zip archive: {error}"))?;
    for name in [kind.name(), "geoip.dat", "geosite.dat"] {
        let mut source = archive
            .by_name(name)
            .map_err(|error| format!("archive is missing {name}: {error}"))?;
        let mut target = fs::File::create(destination.join(name))
            .map_err(|error| format!("could not create {name}: {error}"))?;
        std::io::copy(&mut source, &mut target)
            .map_err(|error| format!("could not extract {name}: {error}"))?;
        target.flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn extract_sing_box(bytes: &[u8], destination: &Path) -> Result<(), String> {
    let decoder = GzDecoder::new(Cursor::new(bytes));
    let mut archive = tar::Archive::new(decoder);
    for entry in archive
        .entries()
        .map_err(|error| format!("invalid tar archive: {error}"))?
    {
        let mut entry = entry.map_err(|error| format!("invalid tar entry: {error}"))?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path().map_err(|error| error.to_string())?;
        if path.file_name().and_then(|name| name.to_str()) != Some("sing-box") {
            continue;
        }
        let mut target = fs::File::create(destination.join("sing-box"))
            .map_err(|error| format!("could not create sing-box: {error}"))?;
        std::io::copy(&mut entry, &mut target)
            .map_err(|error| format!("could not extract sing-box: {error}"))?;
        target.flush().map_err(|error| error.to_string())?;
        return Ok(());
    }
    Err("archive is missing sing-box".to_string())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)
        .map_err(|error| format!("could not inspect staged binary: {error}"))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .map_err(|error| format!("could not make staged binary executable: {error}"))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), String> {
    Err("managed core installation requires a Unix platform".to_string())
}

fn replace_directory(staged: &Path, target: &Path) -> Result<(), String> {
    let parent = target
        .parent()
        .ok_or_else(|| "managed core path has no parent".to_string())?;
    let backup = parent.join(format!(
        ".{}-backup-{}",
        target.file_name().unwrap_or_default().to_string_lossy(),
        uuid::Uuid::new_v4()
    ));
    let had_target = target.exists();
    if had_target {
        fs::rename(target, &backup)
            .map_err(|error| format!("could not stage existing core for replacement: {error}"))?;
    }
    if let Err(error) = fs::rename(staged, target) {
        if had_target {
            let _ = fs::rename(&backup, target);
        }
        return Err(format!("could not activate managed core: {error}"));
    }
    if had_target {
        let _ = fs::remove_dir_all(backup);
    }
    Ok(())
}

#[cfg(unix)]
fn ensure_cli_link(
    kind: CoreKind,
    binary_path: &Path,
    managed_root: &Path,
) -> Result<Option<String>, String> {
    let bin_dir = platform::home_dir()
        .map(|home| home.join(".local").join("bin"))
        .ok_or_else(|| "could not determine ~/.local/bin".to_string())?;
    ensure_cli_link_in(kind, binary_path, managed_root, &bin_dir)
}

#[cfg(unix)]
fn ensure_cli_link_in(
    kind: CoreKind,
    binary_path: &Path,
    managed_root: &Path,
    bin_dir: &Path,
) -> Result<Option<String>, String> {
    use std::os::unix::fs::symlink;

    fs::create_dir_all(bin_dir)
        .map_err(|error| format!("could not create CLI directory: {error}"))?;
    let destination = bin_dir.join(kind.name());
    if let Ok(metadata) = fs::symlink_metadata(&destination) {
        let replaceable = metadata.file_type().is_symlink()
            && fs::read_link(&destination)
                .ok()
                .is_some_and(|target| target.starts_with(managed_root));
        if !replaceable {
            return Ok(Some(format!(
                "left existing {} untouched",
                destination.display()
            )));
        }
    }

    let temporary = bin_dir.join(format!(".{}-xrat-{}", kind.name(), uuid::Uuid::new_v4()));
    symlink(binary_path, &temporary)
        .map_err(|error| format!("could not create CLI link: {error}"))?;
    if let Err(error) = fs::rename(&temporary, &destination) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("could not activate CLI link: {error}"));
    }
    Ok((!platform::dir_in_path(bin_dir)).then(|| {
        format!(
            "{} is not on PATH; add it to use {} from the shell",
            bin_dir.display(),
            kind.name()
        )
    }))
}

#[cfg(not(unix))]
fn ensure_cli_link(
    _kind: CoreKind,
    _binary_path: &Path,
    _managed_root: &Path,
) -> Result<Option<String>, String> {
    Err("managed core installation requires a Unix platform".to_string())
}

fn binary_version(path: &Path) -> Option<Version> {
    let output = Command::new(path).arg("version").output().ok()?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    parse_version(&text)
}

fn parse_version(text: &str) -> Option<Version> {
    text.split(|character: char| {
        !(character.is_ascii_alphanumeric()
            || character == '.'
            || character == '-'
            || character == '+')
    })
    .filter_map(|token| Version::parse(token.trim_start_matches('v')).ok())
    .next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn serve_once(response: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 1024];
            let _ = socket.read(&mut request).await;
            socket.write_all(response).await.unwrap();
        });
        format!("http://{address}/core")
    }

    fn test_release(url: String) -> CoreRelease {
        CoreRelease {
            version: Version::new(5, 52, 0),
            tag: "v5.52.0".to_string(),
            asset: ReleaseAsset {
                name: "v2ray.zip".to_string(),
                url,
                sha256: "0".repeat(64),
            },
        }
    }

    #[tokio::test]
    async fn streams_downloads_with_known_and_unknown_lengths() {
        let client = reqwest::Client::new();
        let responses: [(&[u8], &[u8]); 2] = [
            (
                b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\nConnection: close\r\n\r\nhello world",
                b"hello world",
            ),
            (
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n",
                b"hello world",
            ),
        ];

        for (response, expected) in responses {
            let release = test_release(serve_once(response).await);
            let bytes = download_archive(&client, CoreKind::V2Ray, &release, false)
                .await
                .unwrap();
            assert_eq!(bytes, expected);
        }
    }

    #[test]
    fn parses_versions_from_supported_core_output() {
        assert_eq!(
            parse_version("Xray 26.3.27 (Xray, Penetrates Everything.)"),
            Some(Version::new(26, 3, 27))
        );
        assert_eq!(
            parse_version("sing-box version 1.13.18"),
            Some(Version::new(1, 13, 18))
        );
        assert_eq!(
            parse_version("V2Ray 5.52.0 (V2Fly)"),
            Some(Version::new(5, 52, 0))
        );
    }

    #[test]
    fn builds_latest_and_pinned_release_api_urls() {
        assert_eq!(
            release_api_url(CoreKind::Xray, None, false),
            "https://api.github.com/repos/XTLS/Xray-core/releases/latest"
        );
        assert_eq!(
            release_api_url(CoreKind::SingBox, Some(&Version::new(1, 13, 2)), false,),
            "https://api.github.com/repos/SagerNet/sing-box/releases/tags/v1.13.2"
        );
        assert_eq!(
            release_api_url(CoreKind::V2Ray, Some(&Version::new(5, 52, 0)), false),
            "https://api.github.com/repos/v2fly/v2ray-core/releases/tags/v5.52.0"
        );
        assert_eq!(
            release_api_url(CoreKind::Xray, None, true),
            "https://api.github.com/repos/XTLS/Xray-core/releases?per_page=100"
        );
    }

    #[test]
    fn selects_newest_published_prerelease_by_creation_time() {
        let releases = [
            ("v26.3.27", false, "2026-03-27T00:00:00Z"),
            ("v26.6.1", true, "2026-06-01T00:00:00Z"),
            ("v26.7.28", true, "2026-07-28T00:00:00Z"),
        ]
        .into_iter()
        .map(|(tag, prerelease, created_at)| GithubRelease {
            tag_name: tag.to_string(),
            prerelease,
            created_at: created_at.to_string(),
            assets: Vec::new(),
        })
        .collect();

        let selected = newest_prerelease(releases).unwrap();

        assert_eq!(selected.tag_name, "v26.7.28");
    }

    #[test]
    fn rejects_missing_or_invalid_release_digest() {
        assert!(parse_sha256(None).is_err());
        assert!(parse_sha256(Some("sha256:nope")).is_err());
        assert!(parse_sha256(Some("sha512:abcd")).is_err());
    }

    #[test]
    fn rejects_checksum_mismatch() {
        let asset = ReleaseAsset {
            name: "core.zip".to_string(),
            url: "https://example.invalid/core.zip".to_string(),
            sha256: "0".repeat(64),
        };
        assert!(verify_checksum(&asset, b"different").is_err());
    }

    #[test]
    fn release_metadata_requires_the_platform_asset_digest() {
        let version = Version::new(1, 13, 18);
        let name = asset_name(CoreKind::SingBox, &version).unwrap();
        let payload = GithubRelease {
            tag_name: "v1.13.18".to_string(),
            prerelease: false,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            assets: vec![GithubAsset {
                name,
                browser_download_url: "https://example.invalid/sing-box.tar.gz".to_string(),
                digest: None,
            }],
        };
        assert!(release_from_payload(CoreKind::SingBox, payload).is_err());
    }

    #[test]
    fn detects_outdated_installed_core() {
        let probe = CoreProbe {
            kind: CoreKind::V2Ray,
            path: Some(PathBuf::from("/usr/bin/v2ray")),
            version: Some(Version::new(5, 48, 0)),
            managed: false,
            latest: Ok(CoreRelease {
                version: Version::new(5, 52, 0),
                tag: "v5.52.0".to_string(),
                asset: ReleaseAsset {
                    name: "v2ray.zip".to_string(),
                    url: "https://example.invalid/v2ray.zip".to_string(),
                    sha256: "0".repeat(64),
                },
            }),
        };
        assert!(probe.outdated());
        assert!(probe.detail().contains("latest v5.52.0; external"));
    }

    #[test]
    fn selects_supported_platform_assets() {
        let version = Version::new(1, 13, 18);
        let cases = [
            (CoreKind::Xray, "linux", "x86_64", "Xray-linux-64.zip"),
            (
                CoreKind::Xray,
                "linux",
                "aarch64",
                "Xray-linux-arm64-v8a.zip",
            ),
            (CoreKind::Xray, "macos", "x86_64", "Xray-macos-64.zip"),
            (
                CoreKind::Xray,
                "macos",
                "aarch64",
                "Xray-macos-arm64-v8a.zip",
            ),
            (CoreKind::V2Ray, "linux", "x86_64", "v2ray-linux-64.zip"),
            (
                CoreKind::V2Ray,
                "linux",
                "aarch64",
                "v2ray-linux-arm64-v8a.zip",
            ),
            (CoreKind::V2Ray, "macos", "x86_64", "v2ray-macos-64.zip"),
            (
                CoreKind::V2Ray,
                "macos",
                "aarch64",
                "v2ray-macos-arm64-v8a.zip",
            ),
            (
                CoreKind::SingBox,
                "linux",
                "x86_64",
                "sing-box-1.13.18-linux-amd64.tar.gz",
            ),
            (
                CoreKind::SingBox,
                "linux",
                "aarch64",
                "sing-box-1.13.18-linux-arm64.tar.gz",
            ),
            (
                CoreKind::SingBox,
                "macos",
                "x86_64",
                "sing-box-1.13.18-darwin-amd64.tar.gz",
            ),
            (
                CoreKind::SingBox,
                "macos",
                "aarch64",
                "sing-box-1.13.18-darwin-arm64.tar.gz",
            ),
        ];
        for (kind, os, arch, expected) in cases {
            assert_eq!(asset_name_for(kind, &version, os, arch).unwrap(), expected);
        }
        assert!(asset_name_for(CoreKind::Xray, &version, "windows", "x86_64").is_err());
    }

    #[test]
    fn replaces_managed_directory_after_staging() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("xray");
        let staged = root.path().join("staged");
        fs::create_dir(&target).unwrap();
        fs::create_dir(&staged).unwrap();
        fs::write(target.join("xray"), "old").unwrap();
        fs::write(staged.join("xray"), "new").unwrap();

        replace_directory(&staged, &target).unwrap();

        assert_eq!(fs::read_to_string(target.join("xray")).unwrap(), "new");
        assert!(!staged.exists());
    }

    #[test]
    fn restores_existing_directory_when_activation_fails() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("xray");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("xray"), "old").unwrap();

        assert!(replace_directory(&root.path().join("missing"), &target).is_err());

        assert_eq!(fs::read_to_string(target.join("xray")).unwrap(), "old");
    }

    #[cfg(unix)]
    #[test]
    fn leaves_unmanaged_cli_entry_untouched() {
        let root = tempfile::tempdir().unwrap();
        let managed_root = root.path().join("cores");
        let binary = managed_root.join("xray").join("xray");
        let bin_dir = root.path().join("bin");
        fs::create_dir_all(binary.parent().unwrap()).unwrap();
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(&binary, "managed").unwrap();
        fs::write(bin_dir.join("xray"), "external").unwrap();

        let warning = ensure_cli_link_in(CoreKind::Xray, &binary, &managed_root, &bin_dir)
            .unwrap()
            .expect("collision should warn");

        assert!(warning.contains("left existing"));
        assert_eq!(
            fs::read_to_string(bin_dir.join("xray")).unwrap(),
            "external"
        );
    }

    #[cfg(unix)]
    #[test]
    fn creates_cli_link_to_managed_binary() {
        let root = tempfile::tempdir().unwrap();
        let managed_root = root.path().join("cores");
        let binary = managed_root.join("xray").join("xray");
        let bin_dir = root.path().join("bin");
        fs::create_dir_all(binary.parent().unwrap()).unwrap();
        fs::write(&binary, "managed").unwrap();

        ensure_cli_link_in(CoreKind::Xray, &binary, &managed_root, &bin_dir).unwrap();

        assert_eq!(fs::read_link(bin_dir.join("xray")).unwrap(), binary);
    }

    #[test]
    fn extracts_only_expected_xray_files() {
        let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default();
        for (name, value) in [
            ("xray", "binary"),
            ("geoip.dat", "ip"),
            ("geosite.dat", "site"),
            ("../escape", "bad"),
        ] {
            archive.start_file(name, options).unwrap();
            archive.write_all(value.as_bytes()).unwrap();
        }
        let bytes = archive.finish().unwrap().into_inner();
        let root = tempfile::tempdir().unwrap();

        extract_zip(CoreKind::Xray, &bytes, root.path()).unwrap();

        assert_eq!(
            fs::read_to_string(root.path().join("xray")).unwrap(),
            "binary"
        );
        assert!(!root.path().parent().unwrap().join("escape").exists());
    }

    #[test]
    fn extracts_nested_sing_box_binary() {
        let encoder = GzEncoder::new(Vec::new(), Compression::default());
        let mut archive = tar::Builder::new(encoder);
        let contents = b"sing-box-binary";
        let mut header = tar::Header::new_gnu();
        header.set_size(contents.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        archive
            .append_data(
                &mut header,
                "sing-box-1.0.0-linux-amd64/sing-box",
                &contents[..],
            )
            .unwrap();
        let encoder = archive.into_inner().unwrap();
        let bytes = encoder.finish().unwrap();
        let root = tempfile::tempdir().unwrap();

        extract_sing_box(&bytes, root.path()).unwrap();

        assert_eq!(fs::read(root.path().join("sing-box")).unwrap(), contents);
    }
}
