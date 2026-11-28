use super::prelude::*;
use super::release::*;

pub(crate) async fn install(
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

pub(crate) async fn install_inner(
    context: &AppContext,
    kind: CoreKind,
    release: &CoreRelease,
    progress_enabled: bool,
) -> Result<InstallResult, String> {
    let client = xrat_support::http::Client::builder()
        .timeout(Duration::from_secs(120))
        .user_agent(concat!("xrat/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| error.to_string())?;
    let bytes = download_archive(&client, kind, release, progress_enabled).await?;
    install_archive(context, kind, release, &bytes)
}

pub(crate) async fn download_archive(
    client: &xrat_support::http::Client,
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

pub(crate) fn install_archive(
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

    let staged_binary = payload.join(binary_file_name(kind));
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
    let binary_path = target.join(binary_file_name(kind));
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

pub(crate) fn verify_checksum(asset: &ReleaseAsset, bytes: &[u8]) -> Result<(), String> {
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual == asset.sha256 {
        return Ok(());
    }
    Err(format!(
        "SHA-256 mismatch for {} (expected {}, got {actual})",
        asset.name, asset.sha256
    ))
}

pub(crate) fn managed_root() -> Result<PathBuf, String> {
    platform::xdg_data_home()
        .map(|path| path.join("xrat").join("cores"))
        .ok_or_else(|| "could not determine the user data directory".to_string())
}

pub(crate) fn extract_archive(
    kind: CoreKind,
    bytes: &[u8],
    destination: &Path,
) -> Result<(), String> {
    extract_archive_for(kind, bytes, destination, cfg!(windows))
}

pub(crate) fn extract_archive_for(
    kind: CoreKind,
    bytes: &[u8],
    destination: &Path,
    windows: bool,
) -> Result<(), String> {
    let wanted = binary_file_name_for(kind, windows);
    match kind {
        CoreKind::Xray | CoreKind::V2Ray => extract_zip_named(kind, bytes, destination, &wanted),
        CoreKind::SingBox if windows => extract_zip_named(kind, bytes, destination, &wanted),
        CoreKind::SingBox => extract_sing_box_named(bytes, destination, &wanted),
    }
}

pub(crate) fn extract_zip_named(
    kind: CoreKind,
    bytes: &[u8],
    destination: &Path,
    wanted: &str,
) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("invalid zip archive: {error}"))?;

    // Entries may sit inside a versioned folder, so match on the file name
    // rather than the full archive path.
    let index = zip_binary_index(&mut archive, kind, wanted)?;
    {
        let mut source = archive
            .by_index(index)
            .map_err(|error| format!("could not read archive entry: {error}"))?;
        let mut target = fs::File::create(destination.join(wanted))
            .map_err(|error| format!("could not create {wanted}: {error}"))?;
        std::io::copy(&mut source, &mut target)
            .map_err(|error| format!("could not extract {wanted}: {error}"))?;
        target.flush().map_err(|error| error.to_string())?;
    }

    // geoip.dat / geosite.dat are optional: not every upstream release ships
    // them, and the runtime fetches them separately when they are missing.
    for name in ["geoip.dat", "geosite.dat"] {
        let mut found = None;
        for index in 0..archive.len() {
            let source = archive
                .by_index(index)
                .map_err(|error| format!("could not read archive entry: {error}"))?;
            if !source.is_file() {
                continue;
            }
            let entry_name = source
                .name()
                .map_err(|error| format!("could not read archive entry name: {error}"))?;
            let file_name = entry_name.rsplit(['/', '\\']).next().unwrap_or_default();
            if file_name == name {
                found = Some(index);
                break;
            }
        }
        let Some(index) = found else {
            continue;
        };
        {
            let mut source = archive
                .by_index(index)
                .map_err(|error| format!("could not read archive entry: {error}"))?;
            let mut target = fs::File::create(destination.join(name))
                .map_err(|error| format!("could not create {name}: {error}"))?;
            std::io::copy(&mut source, &mut target)
                .map_err(|error| format!("could not extract {name}: {error}"))?;
            target.flush().map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}

fn zip_binary_index(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    kind: CoreKind,
    wanted: &str,
) -> Result<usize, String> {
    let bare = kind.name();
    let mut exact = None;
    let mut fallback = None;
    for index in 0..archive.len() {
        let source = archive
            .by_index(index)
            .map_err(|error| format!("could not read archive entry: {error}"))?;
        if !source.is_file() {
            continue;
        }
        let entry_name = source
            .name()
            .map_err(|error| format!("could not read archive entry name: {error}"))?;
        let file_name = entry_name.rsplit(['/', '\\']).next().unwrap_or_default();
        if file_name == wanted {
            exact = Some(index);
            break;
        }
        if fallback.is_none() && file_name == bare {
            fallback = Some(index);
        }
    }
    exact
        .or(fallback)
        .ok_or_else(|| format!("archive is missing {wanted}: specified file not found in archive"))
}

#[cfg(any(windows, test))]
pub(crate) fn prepare_machine_binary(
    kind: CoreKind,
    release: &CoreRelease,
    bytes: &[u8],
) -> Result<Vec<u8>, String> {
    use std::io::Read;
    const LIMIT: u64 = 256 * 1024 * 1024;
    verify_checksum(&release.asset, bytes)?;
    let wanted = binary_file_name_for(kind, true);
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|error| error.to_string())?;
    let index = zip_binary_index(&mut archive, kind, &wanted)?;
    let source = archive.by_index(index).map_err(|error| error.to_string())?;
    if source.size() > LIMIT {
        return Err("machine core executable exceeds 256 MiB".into());
    }
    let mut output = Vec::new();
    source
        .take(LIMIT + 1)
        .read_to_end(&mut output)
        .map_err(|error| error.to_string())?;
    if output.is_empty() || output.len() as u64 > LIMIT {
        return Err("machine core executable is empty or exceeds 256 MiB".into());
    }
    Ok(output)
}

pub(crate) fn extract_sing_box_named(
    bytes: &[u8],
    destination: &Path,
    wanted: &str,
) -> Result<(), String> {
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
        if path.file_name().and_then(|name| name.to_str()) != Some(wanted) {
            continue;
        }
        let mut target = fs::File::create(destination.join(wanted))
            .map_err(|error| format!("could not create {wanted}: {error}"))?;
        std::io::copy(&mut entry, &mut target)
            .map_err(|error| format!("could not extract {wanted}: {error}"))?;
        target.flush().map_err(|error| error.to_string())?;
        return Ok(());
    }
    Err(format!("archive is missing {wanted}"))
}

#[cfg(unix)]
pub(crate) fn set_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)
        .map_err(|error| format!("could not inspect staged binary: {error}"))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)
        .map_err(|error| format!("could not make staged binary executable: {error}"))
}

#[cfg(not(unix))]
pub(crate) fn set_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}

/// On-disk file name of a core binary. Windows executables carry the `.exe`
/// suffix, and the upstream archives ship them under that name too.
pub(crate) fn binary_file_name(kind: CoreKind) -> String {
    binary_file_name_for(kind, cfg!(windows))
}

/// On-disk file name of a core binary for the given platform. Windows
/// executables carry the `.exe` suffix, and the upstream archives ship them
/// under that name too.
pub(crate) fn binary_file_name_for(kind: CoreKind, windows: bool) -> String {
    if windows {
        format!("{}.exe", kind.name())
    } else {
        kind.name().to_string()
    }
}

pub(crate) fn replace_directory(staged: &Path, target: &Path) -> Result<(), String> {
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
pub(crate) fn ensure_cli_link(
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
pub(crate) fn ensure_cli_link_in(
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
pub(crate) fn ensure_cli_link(
    _kind: CoreKind,
    _binary_path: &Path,
    _managed_root: &Path,
) -> Result<Option<String>, String> {
    // Unix links the core into ~/.local/bin. Windows has no equivalent
    // symlink convention, so the managed core is referenced by its absolute
    // path in config.toml instead.
    Ok(None)
}

pub(crate) fn binary_version(path: &Path) -> Option<Version> {
    binary_version_with_spawner(
        path,
        std::sync::Arc::new(xrat_support::process::SystemProcessSpawner),
    )
}

pub(crate) fn binary_version_with_spawner(
    path: &Path,
    spawner: std::sync::Arc<dyn xrat_support::process::ProcessSpawner>,
) -> Option<Version> {
    let output = Command::with_spawner(path, spawner.clone())
        .arg("version")
        .output()
        .ok()?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    parse_version(&text)
}

pub(crate) fn parse_version(text: &str) -> Option<Version> {
    text.split(|character: char| {
        !(character.is_ascii_alphanumeric()
            || character == '.'
            || character == '-'
            || character == '+')
    })
    .filter_map(|token| Version::parse(token.trim_start_matches('v')).ok())
    .next()
}

pub(crate) fn configured_path(context: &AppContext, kind: CoreKind) -> &Path {
    match kind {
        CoreKind::Xray => &context.runtime_paths.xray_path,
        CoreKind::SingBox => &context.runtime_paths.sing_box_path,
        CoreKind::V2Ray => &context.runtime_paths.v2ray_path,
    }
}

pub(crate) fn resolve_installed_path(configured: &Path) -> Option<PathBuf> {
    if configured.is_file() {
        return Some(configured.to_path_buf());
    }
    if configured.components().count() == 1 {
        return configured.to_str().and_then(platform::binary_on_path);
    }
    None
}

pub(crate) async fn probe_all(context: &AppContext) -> Vec<CoreProbe> {
    let client = xrat_support::http::Client::builder()
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
