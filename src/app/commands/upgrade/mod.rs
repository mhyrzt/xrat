mod release;
mod source;

use std::path::{Path, PathBuf};

use crate::app::AppError;
use crate::app::context::AppContext;
use crate::cli::UpgradeArgs;

pub(crate) const REPO: &str = "mhyrzt/xrat";

pub async fn run(_context: &AppContext, args: &UpgradeArgs) -> crate::app::Result<()> {
    let target = current_exe()?;

    if args.source {
        source::upgrade(args, &target).await
    } else {
        release::upgrade(args, &target).await
    }
}

fn current_exe() -> crate::app::Result<PathBuf> {
    std::env::current_exe().map_err(|error| {
        AppError::InvalidArgument(format!(
            "could not resolve the running xrat binary: {error}"
        ))
    })
}

/// Atomically replace the running binary at `target` with `new_binary`.
///
/// The new file is staged in the same directory so the final rename stays on a
/// single filesystem; on Linux replacing a running executable this way is safe.
pub(crate) fn install_binary(new_binary: &Path, target: &Path) -> crate::app::Result<()> {
    let dir = target.parent().unwrap_or_else(|| Path::new("."));
    let mut staged = tempfile::NamedTempFile::new_in(dir).map_err(|error| {
        AppError::InvalidArgument(format!(
            "cannot stage upgrade in {} (permission?): {error}",
            dir.display()
        ))
    })?;
    std::io::copy(&mut std::fs::File::open(new_binary)?, staged.as_file_mut())?;
    staged.as_file().sync_all()?;
    set_executable(staged.path())?;

    staged.persist(target).map_err(|error| {
        AppError::InvalidArgument(format!(
            "cannot replace {} (permission?): {}",
            target.display(),
            error.error
        ))
    })?;

    Ok(())
}

fn set_executable(path: &Path) -> crate::app::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

/// Compare release tags ignoring a leading `v`, e.g. `v0.2.1` == `0.2.1`.
pub(crate) fn same_version(left: &str, right: &str) -> bool {
    normalize_tag(left) == normalize_tag(right)
}

fn normalize_tag(tag: &str) -> &str {
    tag.trim().trim_start_matches('v')
}

pub(crate) fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_version_ignores_v_prefix() {
        assert!(same_version("v0.2.1", "0.2.1"));
        assert!(same_version("0.2.1", "v0.2.1"));
        assert!(same_version(" v0.2.1 ", "0.2.1"));
        assert!(!same_version("v0.2.1", "0.2.2"));
    }
}
