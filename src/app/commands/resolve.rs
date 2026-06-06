//! Resolve user-supplied config/subscription identifiers that may be either a
//! numeric id (legacy) or a short ref prefix (`xrat connect a1b2`).

use crate::app::AppError;
use crate::app::context::AppContext;
use crate::db::RefMatch;
use crate::support::refs::is_ref_prefix;

/// Resolve a config identifier: a numeric id is tried first (transition
/// compatibility), then a ref prefix.
pub async fn resolve_config_id(context: &AppContext, raw: &str) -> crate::app::Result<i64> {
    if let Some(id) = existing_config_numeric_id(context, raw).await? {
        return Ok(id);
    }
    if is_ref_prefix(raw) {
        match context.db.resolve_config_ref_prefix(raw).await? {
            RefMatch::Unique(id) => return Ok(id),
            RefMatch::Ambiguous => {
                return Err(AppError::InvalidArgument(format!(
                    "config ref prefix '{raw}' is ambiguous; provide more characters"
                )));
            }
            RefMatch::None => {}
        }
    }
    Err(AppError::InvalidArgument(format!(
        "no config found for '{raw}'"
    )))
}

/// Resolve a subscription identifier: numeric id first, then a ref prefix.
pub async fn resolve_subscription_id(context: &AppContext, raw: &str) -> crate::app::Result<i64> {
    if let Some(id) = existing_subscription_numeric_id(context, raw).await? {
        return Ok(id);
    }
    if is_ref_prefix(raw) {
        match context.db.resolve_subscription_ref_prefix(raw).await? {
            RefMatch::Unique(id) => return Ok(id),
            RefMatch::Ambiguous => {
                return Err(AppError::InvalidArgument(format!(
                    "subscription ref prefix '{raw}' is ambiguous; provide more characters"
                )));
            }
            RefMatch::None => {}
        }
    }
    Err(AppError::InvalidArgument(format!(
        "no subscription found for '{raw}'"
    )))
}

async fn existing_config_numeric_id(
    context: &AppContext,
    raw: &str,
) -> crate::app::Result<Option<i64>> {
    let Ok(id) = raw.parse::<i64>() else {
        return Ok(None);
    };
    Ok(context
        .db
        .get_config_by_id(id)
        .await?
        .is_some()
        .then_some(id))
}

async fn existing_subscription_numeric_id(
    context: &AppContext,
    raw: &str,
) -> crate::app::Result<Option<i64>> {
    let Ok(id) = raw.parse::<i64>() else {
        return Ok(None);
    };
    Ok(context
        .db
        .get_subscription_by_id(id)
        .await?
        .is_some()
        .then_some(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::config::AppConfig;
    use crate::app::context::RuntimePaths;
    use crate::db::{Database, DatabaseConnectionConfig, ImportSource, SourceKind};
    use crate::model::{Node, Protocol};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn resolves_numeric_config_id_and_ref_prefix() {
        let context = test_context("config").await;
        let config = seed_config(&context).await;

        let by_id = resolve_config_id(&context, &config.id.to_string())
            .await
            .expect("numeric id should resolve");
        let by_ref = resolve_config_id(&context, &config.r#ref[..8])
            .await
            .expect("ref prefix should resolve");

        assert_eq!(by_id, config.id);
        assert_eq!(by_ref, config.id);
    }

    #[tokio::test]
    async fn resolves_numeric_subscription_id_and_ref_prefix() {
        let context = test_context("subscription").await;
        let subscription = seed_subscription(&context).await;

        let by_id = resolve_subscription_id(&context, &subscription.id.to_string())
            .await
            .expect("numeric id should resolve");
        let by_ref = resolve_subscription_id(&context, &subscription.r#ref[..8])
            .await
            .expect("ref prefix should resolve");

        assert_eq!(by_id, subscription.id);
        assert_eq!(by_ref, subscription.id);
    }

    #[tokio::test]
    async fn missing_identifier_returns_invalid_argument() {
        let context = test_context("missing").await;
        let err = resolve_config_id(&context, "missing")
            .await
            .expect_err("missing config should error");

        assert!(matches!(err, AppError::InvalidArgument(_)));
    }

    async fn seed_config(context: &AppContext) -> crate::db::ConfigRecord {
        seed_subscription(context).await;
        context
            .db
            .list_configs(&Default::default())
            .await
            .expect("configs should load")
            .into_iter()
            .next()
            .expect("config should exist")
    }

    async fn seed_subscription(context: &AppContext) -> crate::db::SubscriptionRecord {
        context
            .db
            .import_nodes(
                &ImportSource {
                    kind: SourceKind::RawText,
                    value: "seed".to_string(),
                    name: Some("seed".to_string()),
                },
                &[Node {
                    protocol: Protocol::Vless,
                    address: "example.com".to_string(),
                    port: 443,
                    username: None,
                    uuid: Some("00000000-0000-0000-0000-000000000001".to_string()),
                    password: None,
                    method: None,
                    network: "tcp".to_string(),
                    tls: Some("tls".to_string()),
                    sni: Some("example.com".to_string()),
                    host: None,
                    path: None,
                    name: Some("seed".to_string()),
                    extensions: None,
                    raw_config: "vless://00000000-0000-0000-0000-000000000001@example.com:443#seed"
                        .to_string(),
                }],
            )
            .await
            .expect("import should succeed");

        context
            .db
            .list_subscriptions()
            .await
            .expect("subscriptions should load")
            .into_iter()
            .next()
            .expect("subscription should exist")
    }

    async fn test_context(prefix: &str) -> AppContext {
        let root = std::env::temp_dir().join(format!(
            "xrat-resolve-{prefix}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("root should be created");
        let database_config = DatabaseConnectionConfig::Sqlite {
            path: root.join("db.sqlite"),
        };
        let db = Database::connect(&database_config)
            .await
            .expect("database should connect");
        AppContext {
            db,
            app_config: AppConfig::default(),
            runtime_paths: RuntimePaths {
                root_dir: root.clone(),
                database_config,
                database_path: root.join("db.sqlite"),
                database_label: root.join("db.sqlite").display().to_string(),
                config_path: root.join("config.toml"),
                runtime_dir: root.join("runtime"),
                xray_path: "xray".into(),
                v2ray_path: "v2ray".into(),
                sing_box_path: "sing-box".into(),
            },
        }
    }
}
