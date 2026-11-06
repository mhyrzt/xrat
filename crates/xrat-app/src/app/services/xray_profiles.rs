use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::app::{AppError, Result, context::AppContext};
use sha2::{Digest, Sha256};
use xrat_config::XrayDocument;
use xrat_db::{ConfigRecord, XrayProfileRecord};

#[derive(Clone)]
pub(crate) struct ProfileStore {
    root: PathBuf,
}

impl ProfileStore {
    pub(crate) fn for_context(context: &AppContext) -> Self {
        let identity = match &context.runtime_paths.database_config {
            xrat_db::DatabaseConnectionConfig::Sqlite { path } => std::fs::canonicalize(path)
                .unwrap_or_else(|_| path.clone())
                .to_string_lossy()
                .into_owned(),
            xrat_db::DatabaseConnectionConfig::Postgres { .. } => {
                context.runtime_paths.database_label.clone()
            }
        };
        Self {
            root: context
                .runtime_paths
                .root_dir
                .join("profiles")
                .join(digest(identity.as_bytes())),
        }
    }

    pub(crate) fn lock(&self) -> Result<File> {
        private_directory(self.root.parent().expect("profile parent"))?;
        private_directory(&self.root)?;
        let path = self.root.join(".lock");
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = match options.open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                regular_file(&path)?;
                OpenOptions::new().read(true).write(true).open(&path)?
            }
            Err(error) => return Err(error.into()),
        };
        regular_file(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let opened = file.metadata()?;
            let current = fs::symlink_metadata(&path)?;
            if opened.ino() != current.ino() || opened.dev() != current.dev() {
                return Err(AppError::InvalidArgument(
                    "Xray profile lock changed while opening it".into(),
                ));
            }
        }
        file.try_lock().map_err(|_| {
            AppError::InvalidArgument("Xray profile storage is busy; retry the operation".into())
        })?;
        Ok(file)
    }

    pub(crate) fn lock_if_present(&self) -> Result<Option<File>> {
        match fs::symlink_metadata(&self.root) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
            Ok(_) => self.lock().map(Some),
        }
    }

    fn path(&self, key: &str) -> Result<PathBuf> {
        if key.len() != 64
            || !key
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(AppError::InvalidArgument(
                "invalid Xray profile file identifier".into(),
            ));
        }
        Ok(self.root.join(format!("{key}.json")))
    }

    fn read_bytes(&self, key: &str) -> Result<Vec<u8>> {
        for directory in [self.root.parent(), Some(self.root.as_path())]
            .into_iter()
            .flatten()
        {
            if !fs::symlink_metadata(directory)?.is_dir() {
                return Err(AppError::InvalidArgument(
                    "unsafe Xray profile directory".into(),
                ));
            }
        }
        let path = self.path(key)?;
        regular_file(&path).map_err(|_| AppError::InvalidArgument("Xray profile file is missing or unsafe; restore the private profiles directory from backup or reimport the original file".into()))?;
        let file = File::open(&path)?;
        let path_metadata = fs::symlink_metadata(&path)?;
        if !path_metadata.is_file() {
            return Err(AppError::InvalidArgument("unsafe Xray profile file".into()));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let opened = file.metadata()?;
            if opened.ino() != path_metadata.ino() || opened.dev() != path_metadata.dev() {
                return Err(AppError::InvalidArgument(
                    "Xray profile changed while opening it".into(),
                ));
            }
        }
        let mut bytes = Vec::new();
        file.take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(AppError::InvalidArgument(
                "Xray profile exceeds 4 MiB".into(),
            ));
        }
        if digest(&bytes) != key {
            return Err(AppError::InvalidArgument(
                "stored Xray profile failed its integrity check; reimport the original file".into(),
            ));
        }
        Ok(bytes)
    }

    pub(crate) fn read(&self, metadata: &XrayProfileRecord) -> Result<XrayDocument> {
        let bytes = self.read_bytes(&metadata.digest)?;
        if i64::try_from(bytes.len()).ok() != Some(metadata.byte_length) {
            return Err(AppError::InvalidArgument(
                "Xray profile size does not match its metadata".into(),
            ));
        }
        XrayDocument::parse(&bytes).map_err(|error| AppError::InvalidArgument(error.to_string()))
    }

    pub(crate) fn remove(&self, key: &str) -> Result<()> {
        match fs::remove_file(self.path(key)?) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    pub(crate) fn cleanup(&self, retained: &[String]) -> Result<()> {
        match fs::symlink_metadata(&self.root) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
            Ok(metadata) if !metadata.is_dir() => {
                return Err(AppError::InvalidArgument(
                    "unsafe Xray profile directory".into(),
                ));
            }
            Ok(_) => {}
        }
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let name = entry.file_name();
            if name
                .to_str()
                .is_some_and(|name| name.starts_with(".staged-"))
            {
                fs::remove_file(entry.path())?;
                continue;
            }
            let Some(key) = name.to_str().and_then(|name| name.strip_suffix(".json")) else {
                continue;
            };
            if self.path(key).is_ok() && !retained.iter().any(|digest| digest == key) {
                self.remove(key)?;
            }
        }
        Ok(())
    }

    fn cleanup_existing(&self, retained: &[String]) -> Result<()> {
        let Some(_lock) = self.lock_if_present()? else {
            return Ok(());
        };
        self.cleanup(retained)
    }
}

pub(crate) async fn load(context: &AppContext, config: &ConfigRecord) -> Result<XrayDocument> {
    let store = ProfileStore::for_context(context);
    load_from_database(&context.db, Some(&store), config).await
}

pub(crate) async fn load_from_database(
    database: &xrat_db::Database,
    store: Option<&ProfileStore>,
    config: &ConfigRecord,
) -> Result<XrayDocument> {
    if !config.is_xray_profile() {
        return Err(AppError::InvalidArgument(
            "config is not a complete Xray profile".into(),
        ));
    }
    if !config.raw_config.is_empty() {
        return parse_stored_profile(config, config.raw_config.as_bytes());
    }

    let store = store.ok_or_else(|| {
        AppError::InvalidArgument(
            "Xray profile data is missing; restore the profiles directory from backup or reimport the original file".into(),
        )
    })?;
    let _lock = store.lock()?;
    let metadata = database.get_xray_profile(config.id).await?.ok_or_else(|| {
        AppError::InvalidArgument("Xray profile data is missing; reimport the original file".into())
    })?;
    let document = store.read(&metadata)?;
    let document = parse_stored_profile(config, document.as_bytes())?;
    let raw_config = std::str::from_utf8(document.as_bytes())
        .map_err(|_| AppError::InvalidArgument("Xray profile must contain UTF-8 JSON".into()))?;
    database
        .migrate_legacy_xray_profile(config.id, &metadata.digest, raw_config)
        .await?;
    match database.xray_profile_digests().await {
        Ok(retained) => {
            if let Err(error) = store.cleanup(&retained) {
                tracing::warn!(%error, "failed to remove migrated Xray profile file");
            }
        }
        Err(error) => tracing::warn!(%error, "failed to inspect legacy Xray profile files"),
    }
    Ok(document)
}

fn parse_stored_profile(config: &ConfigRecord, bytes: &[u8]) -> Result<XrayDocument> {
    if config.dedup_key != format!("xray-json:{}", digest(bytes)) {
        return Err(AppError::InvalidArgument(
            "stored Xray profile failed its integrity check; reimport the original file".into(),
        ));
    }
    parse(bytes)?.ok_or_else(|| {
        AppError::InvalidArgument("stored Xray profile is not a complete Xray configuration".into())
    })
}

pub(crate) fn parse(bytes: &[u8]) -> Result<Option<XrayDocument>> {
    let trimmed = bytes
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace());
    if !matches!(trimmed, Some(b'{' | b'[')) {
        return Ok(None);
    }
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(AppError::InvalidArgument(
            "Xray profile exceeds 4 MiB".into(),
        ));
    }
    let document =
        XrayDocument::parse(bytes).map_err(|error| AppError::InvalidArgument(error.to_string()))?;
    let value: serde_json::Value = serde_json::from_slice(document.as_bytes())?;
    if !value
        .get("outbounds")
        .is_some_and(serde_json::Value::is_array)
    {
        return Err(AppError::InvalidArgument("expected a complete Xray configuration with an outbounds array; SIP008 JSON import is unsupported".into()));
    }
    Ok(Some(document))
}

pub(crate) async fn import(
    context: &AppContext,
    source: &xrat_db::ImportSource,
    document: &XrayDocument,
    name: Option<&str>,
) -> Result<xrat_db::ImportSummary> {
    context.require_local_runtime()?;
    if source.kind == xrat_db::SourceKind::Url {
        return Err(AppError::InvalidArgument("complete Xray profiles must be imported from a local file or pasted JSON; automatic URL refresh is unsupported".into()));
    }
    let bytes = document.as_bytes();
    let key = digest(bytes);
    let raw_config = std::str::from_utf8(bytes)
        .map_err(|_| AppError::InvalidArgument("Xray profile must contain UTF-8 JSON".into()))?;
    let source = xrat_db::ImportSource {
        kind: xrat_db::SourceKind::RawText,
        value: format!("xray-profile:{key}"),
        name: name.map(str::to_owned).or_else(|| source.name.clone()),
    };
    let summary = context
        .db
        .import_xray_profile(&source, &key, raw_config, name.or(source.name.as_deref()))
        .await?;
    let store = ProfileStore::for_context(context);
    match context.db.xray_profile_digests().await {
        Ok(retained) => {
            if let Err(error) = store.cleanup_existing(&retained) {
                tracing::warn!(%error, "failed to remove imported legacy Xray profile file");
            }
        }
        Err(error) => tracing::warn!(%error, "failed to inspect legacy Xray profile files"),
    }
    Ok(summary)
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn regular_file(path: &Path) -> std::io::Result<()> {
    if !fs::symlink_metadata(path)?.is_file() {
        return Err(std::io::Error::other("expected a regular file"));
    }
    Ok(())
}

fn private_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        #[cfg(unix)]
        let mut builder = fs::DirBuilder::new();
        #[cfg(not(unix))]
        let builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }
    if !fs::symlink_metadata(path)?.is_dir() {
        return Err(AppError::InvalidArgument(
            "Xray profile storage must be a private directory, not a symlink".into(),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
