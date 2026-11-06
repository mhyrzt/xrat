use crate::connection::DbPool;
use crate::{ImportSource, ImportSummary, XrayProfileRecord};
use sha2::{Digest, Sha256};
use xrat_model::{ConfigId, SubscriptionId};

pub async fn import(
    pool: &DbPool,
    source: &ImportSource,
    digest: &str,
    raw_config: &str,
    name: Option<&str>,
) -> crate::Result<ImportSummary> {
    if raw_config.is_empty()
        || raw_config.len() > 4 * 1024 * 1024
        || digest.len() != 64
        || digest != content_digest(raw_config.as_bytes())
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid Xray profile digest or content",
        )
        .into());
    }
    let reference = xrat_support::refs::generate_ref();
    let subscription_ref = xrat_support::refs::generate_ref();
    let key = format!("xray-json:{digest}");
    macro_rules! persist {
        ($pool:expr, $postgres:expr) => {{
            let mut tx = $pool.begin().await?;
            if $postgres {
                sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
                    .bind(&key).execute(&mut *tx).await?;
            }
            let statement = "SELECT subscription_id FROM configs WHERE dedup_key = ?1";
            let statement = if $postgres { statement.replace('?', "$") } else { statement.into() };
            let existing: Option<Option<SubscriptionId>> = sqlx::query_scalar(&statement).bind(&key).fetch_optional(&mut *tx).await?;
            let subscription_id = match existing.flatten() {
                Some(id) => id,
                None => {
                    let statement = "INSERT INTO subscriptions (ref, source_kind, name) VALUES (?1, 'raw_text', ?2) RETURNING id";
                    let statement = if $postgres { statement.replace('?', "$") } else { statement.into() };
                    sqlx::query_scalar(&statement).bind(&subscription_ref).bind(name.or(source.name.as_deref())).fetch_one(&mut *tx).await?
                }
            };
            let statement = "UPDATE subscriptions SET name = COALESCE(?2, name) WHERE id = ?1";
            let statement = if $postgres { statement.replace('?', "$") } else { statement.into() };
            sqlx::query(&statement).bind(subscription_id).bind(name.or(source.name.as_deref())).execute(&mut *tx).await?;
            let statement = "INSERT INTO configs (ref, subscription_id, dedup_key, protocol, address, port, network, name, raw_config) VALUES (?1, ?2, ?3, 'xray-json', '', 0, '', ?4, ?5) ON CONFLICT (dedup_key) DO UPDATE SET subscription_id = COALESCE(configs.subscription_id, excluded.subscription_id), name = COALESCE(excluded.name, configs.name), raw_config = CASE WHEN configs.raw_config = '' THEN excluded.raw_config ELSE configs.raw_config END, is_deleted = 0, deleted_at = NULL, imported_at = excluded.imported_at, updated_at = excluded.updated_at WHERE configs.raw_config = '' OR configs.raw_config = excluded.raw_config RETURNING id";
            let statement = if $postgres { statement.replace('?', "$") } else { statement.into() };
            let id: Option<ConfigId> = sqlx::query_scalar(&statement)
                .bind(&reference).bind(subscription_id).bind(&key).bind(name.or(source.name.as_deref()))
                .bind(raw_config).fetch_optional(&mut *tx).await?;
            let Some(id) = id else {
                return Err(std::io::Error::other("Xray profile digest conflicts with different stored content").into());
            };
            let statement = "SELECT raw_config FROM configs WHERE id = ?1";
            let statement = if $postgres { statement.replace('?', "$") } else { statement.into() };
            let stored: String = sqlx::query_scalar(&statement).bind(id).fetch_one(&mut *tx).await?;
            if stored != raw_config {
                return Err(std::io::Error::other("Xray profile content does not match its digest").into());
            }
            let statement = "DELETE FROM xray_profiles WHERE config_id = ?1";
            let statement = if $postgres { statement.replace('?', "$") } else { statement.into() };
            sqlx::query(&statement).bind(id).execute(&mut *tx).await?;
            tx.commit().await?;
            subscription_id
        }};
    }
    let subscription_id = match pool {
        DbPool::Sqlite(pool) => persist!(pool, false),
        DbPool::Postgres(pool) => persist!(pool, true),
    };
    Ok(ImportSummary {
        subscription_id,
        imported_configs: 1,
        removed_configs: 0,
        total_configs: super::configs::get_count(pool).await?,
    })
}

pub async fn migrate_legacy_profile(
    pool: &DbPool,
    id: ConfigId,
    digest: &str,
    raw_config: &str,
) -> crate::Result<()> {
    if digest != content_digest(raw_config.as_bytes()) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "legacy Xray profile content does not match its digest",
        )
        .into());
    }
    macro_rules! migrate {
        ($pool:expr, $postgres:expr) => {{
            let mut tx = $pool.begin().await?;
            let statement = "UPDATE configs SET raw_config = ?3 WHERE id = ?1 AND protocol = 'xray-json' AND dedup_key = ?2 AND (raw_config = '' OR raw_config = ?3)";
            let statement = if $postgres { statement.replace('?', "$") } else { statement.into() };
            let updated = sqlx::query(&statement)
                .bind(id)
                .bind(format!("xray-json:{digest}"))
                .bind(raw_config)
                .execute(&mut *tx)
                .await?;
            if updated.rows_affected() == 0 {
                let statement = "SELECT raw_config FROM configs WHERE id = ?1 AND protocol = 'xray-json' AND dedup_key = ?2";
                let statement = if $postgres { statement.replace('?', "$") } else { statement.into() };
                let stored: Option<String> = sqlx::query_scalar(&statement)
                    .bind(id)
                    .bind(format!("xray-json:{digest}"))
                    .fetch_optional(&mut *tx)
                    .await?;
                if stored.as_deref() != Some(raw_config) {
                    return Err(std::io::Error::other("legacy Xray profile does not match its config record").into());
                }
            }
            let statement = "DELETE FROM xray_profiles WHERE config_id = ?1 AND digest = ?2";
            let statement = if $postgres { statement.replace('?', "$") } else { statement.into() };
            sqlx::query(&statement)
                .bind(id)
                .bind(digest)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
        }};
    }
    match pool {
        DbPool::Sqlite(pool) => migrate!(pool, false),
        DbPool::Postgres(pool) => migrate!(pool, true),
    }
    Ok(())
}

fn content_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub async fn get(pool: &DbPool, id: ConfigId) -> crate::Result<Option<XrayProfileRecord>> {
    let row: Option<(ConfigId, String, i64)> = match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query_as(
                "SELECT config_id, digest, byte_length FROM xray_profiles WHERE config_id = ?1",
            )
            .bind(id)
            .fetch_optional(pool)
            .await?
        }
        DbPool::Postgres(pool) => {
            sqlx::query_as(
                "SELECT config_id, digest, byte_length FROM xray_profiles WHERE config_id = $1",
            )
            .bind(id)
            .fetch_optional(pool)
            .await?
        }
    };
    Ok(
        row.map(|(config_id, digest, byte_length)| XrayProfileRecord {
            config_id,
            digest,
            byte_length,
        }),
    )
}

pub async fn digests(pool: &DbPool) -> crate::Result<Vec<String>> {
    Ok(match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query_scalar("SELECT digest FROM xray_profiles")
                .fetch_all(pool)
                .await?
        }
        DbPool::Postgres(pool) => {
            sqlx::query_scalar("SELECT digest FROM xray_profiles")
                .fetch_all(pool)
                .await?
        }
    })
}
