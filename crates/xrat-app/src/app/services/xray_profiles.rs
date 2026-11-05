use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
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

    pub(crate) fn write(&self, document: &XrayDocument) -> Result<(String, bool)> {
        let key = digest(document.as_bytes());
        let path = self.path(&key)?;
        let existed = path.exists();
        if existed {
            regular_file(&path)?;
            if self
                .read_bytes(&key)
                .is_ok_and(|existing| existing == document.as_bytes())
            {
                return Ok((key, false));
            }
        }
        let mut staged = tempfile::Builder::new()
            .prefix(".staged-")
            .tempfile_in(&self.root)?;
        staged.write_all(document.as_bytes())?;
        staged.as_file().sync_all()?;
        if existed {
            staged.persist(path).map_err(|error| error.error)?;
        } else {
            staged
                .persist_noclobber(path)
                .map_err(|error| error.error)?;
        }
        #[cfg(unix)]
        File::open(&self.root)?.sync_all()?;
        Ok((key, !existed))
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
}

pub(crate) async fn load(context: &AppContext, config: &ConfigRecord) -> Result<XrayDocument> {
    let metadata = context
        .db
        .get_xray_profile(config.id)
        .await?
        .ok_or_else(|| AppError::InvalidArgument("Xray profile metadata is missing".into()))?;
    ProfileStore::for_context(context).read(&metadata)
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
    let store = ProfileStore::for_context(context);
    let _lock = store.lock()?;
    store.cleanup(&context.db.xray_profile_digests().await?)?;
    let (key, created) = store.write(document)?;
    let source = xrat_db::ImportSource {
        kind: xrat_db::SourceKind::RawText,
        value: format!("xray-profile:{key}"),
        name: name.map(str::to_owned).or_else(|| source.name.clone()),
    };
    let result = context
        .db
        .import_xray_profile(
            &source,
            &key,
            document.as_bytes().len() as i64,
            name.or(source.name.as_deref()),
        )
        .await;
    match result {
        Ok(summary) => Ok(summary),
        Err(error) => {
            if created
                && context
                    .db
                    .xray_profile_digests()
                    .await
                    .is_ok_and(|digests| !digests.contains(&key))
            {
                store.remove(&key)?;
            }
            Err(error.into())
        }
    }
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
        let mut builder = fs::DirBuilder::new();
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
