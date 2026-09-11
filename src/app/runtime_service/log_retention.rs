use std::collections::HashSet;
use std::io;

use crate::app::context::AppContext;

pub(crate) async fn cleanup(context: &AppContext) {
    if let Err(error) = prune(context).await {
        tracing::warn!(%error, "could not prune old runtime logs");
    }
}

async fn prune(context: &AppContext) -> crate::app::Result<()> {
    let entries = match std::fs::read_dir(&context.runtime_paths.runtime_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    let expired: HashSet<i64> = context
        .db
        .get_expired_runtime_log_session_ids()
        .await?
        .into_iter()
        .collect();
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let Some(session_id) = name.to_str().and_then(log_session_id) else {
            continue;
        };
        if !expired.contains(&session_id) || !entry.file_type()?.is_file() {
            continue;
        }
        if let Err(error) = std::fs::remove_file(entry.path())
            && error.kind() != io::ErrorKind::NotFound
        {
            tracing::warn!(%error, path = %entry.path().display(), "could not remove old runtime log");
        }
    }
    Ok(())
}

fn log_session_id(name: &str) -> Option<i64> {
    let rest = name.strip_prefix("session-")?;
    let (id, suffix) = rest.split_once('.')?;
    if !matches!(
        suffix,
        "out.log" | "err.log" | "singbox.out.log" | "singbox.err.log"
    ) {
        return None;
    }
    let session_id: i64 = id.parse().ok()?;
    (session_id > 0 && session_id.to_string() == id).then_some(session_id)
}

#[cfg(test)]
mod tests;
