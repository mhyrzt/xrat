use crate::connection::DbPool;
use crate::{ImportSource, ImportSummary, XrayProfileRecord};
use xrat_model::{ConfigId, SubscriptionId};

pub async fn import(
    pool: &DbPool,
    source: &ImportSource,
    digest: &str,
    byte_length: i64,
    name: Option<&str>,
) -> crate::Result<ImportSummary> {
    if byte_length <= 0
        || digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid Xray profile digest or length",
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
            let statement = "INSERT INTO configs (ref, subscription_id, dedup_key, protocol, address, port, network, name, raw_config) VALUES (?1, ?2, ?3, 'xray-json', '', 0, '', ?4, '') ON CONFLICT (dedup_key) DO UPDATE SET subscription_id = COALESCE(configs.subscription_id, excluded.subscription_id), name = COALESCE(excluded.name, configs.name), is_deleted = 0, deleted_at = NULL, imported_at = excluded.imported_at, updated_at = excluded.updated_at RETURNING id";
            let statement = if $postgres { statement.replace('?', "$") } else { statement.into() };
            let id: ConfigId = sqlx::query_scalar(&statement)
                .bind(&reference).bind(subscription_id).bind(&key).bind(name.or(source.name.as_deref()))
                .fetch_one(&mut *tx).await?;
            let statement = "INSERT INTO xray_profiles (config_id, digest, byte_length) VALUES (?1, ?2, ?3) ON CONFLICT (config_id) DO NOTHING";
            let statement = if $postgres { statement.replace('?', "$") } else { statement.into() };
            sqlx::query(&statement).bind(id).bind(digest).bind(byte_length).execute(&mut *tx).await?;
            let statement = "SELECT digest, byte_length FROM xray_profiles WHERE config_id = ?1";
            let statement = if $postgres { statement.replace('?', "$") } else { statement.into() };
            let stored: (String, i64) = sqlx::query_as(&statement).bind(id).fetch_one(&mut *tx).await?;
            if stored != (digest.to_owned(), byte_length) {
                return Err(std::io::Error::other("Xray profile metadata does not match its content").into());
            }
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
