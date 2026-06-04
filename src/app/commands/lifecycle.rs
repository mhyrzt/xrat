use crate::app::commands::list::subscription_json;
use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::cli::{
    DeleteArgs, DeleteConfigArgs, DeleteSubscriptionArgs, DeleteTarget, DisableArgs, EnableArgs,
    RestoreArgs, ShowArgs, ShowConfigArgs, ShowSubscriptionArgs, ShowTarget,
};

pub async fn enable(context: &AppContext, args: &EnableArgs) -> crate::app::Result<()> {
    let config = context.db.get_config_by_id(args.id).await?.ok_or_else(|| {
        crate::app::AppError::InvalidArgument(format!("config {} not found", args.id))
    })?;

    if config.is_deleted {
        println!(
            "{}",
            output::notice(
                format!("Config {} is deleted; restore it first", args.id),
                output::color_enabled()
            )
        );
        return Ok(());
    }

    if config.is_enabled {
        println!(
            "{}",
            output::notice(
                format!("Config {} is already enabled", args.id),
                output::color_enabled()
            )
        );
        return Ok(());
    }

    context.db.set_config_enabled(args.id, true).await?;
    println!(
        "{}",
        output::success(
            format!("Enabled config {}", args.id),
            output::color_enabled()
        )
    );
    Ok(())
}

pub async fn disable(context: &AppContext, args: &DisableArgs) -> crate::app::Result<()> {
    let config = context.db.get_config_by_id(args.id).await?.ok_or_else(|| {
        crate::app::AppError::InvalidArgument(format!("config {} not found", args.id))
    })?;

    if config.is_deleted {
        println!(
            "{}",
            output::notice(
                format!("Config {} is deleted; restore it first", args.id),
                output::color_enabled()
            )
        );
        return Ok(());
    }

    if !config.is_enabled {
        println!(
            "{}",
            output::notice(
                format!("Config {} is already disabled", args.id),
                output::color_enabled()
            )
        );
        return Ok(());
    }

    context.db.set_config_enabled(args.id, false).await?;
    println!(
        "{}",
        output::success(
            format!("Disabled config {}", args.id),
            output::color_enabled()
        )
    );
    Ok(())
}

pub async fn delete(context: &AppContext, args: &DeleteArgs) -> crate::app::Result<()> {
    match &args.target {
        DeleteTarget::Config(config_args) => delete_config(context, config_args).await,
        DeleteTarget::Subscription(subscription_args) => {
            delete_subscription(context, subscription_args).await
        }
    }
}

async fn delete_config(context: &AppContext, args: &DeleteConfigArgs) -> crate::app::Result<()> {
    let config = context.db.get_config_by_id(args.id).await?.ok_or_else(|| {
        crate::app::AppError::InvalidArgument(format!("config {} not found", args.id))
    })?;

    if args.hard {
        context.db.hard_delete_config(args.id).await?;
        println!(
            "{}",
            output::success(
                format!("Permanently deleted config {}", args.id),
                output::color_enabled()
            )
        );
    } else {
        if config.is_deleted {
            println!(
                "{}",
                output::notice(
                    format!("Config {} is already deleted", args.id),
                    output::color_enabled()
                )
            );
            return Ok(());
        }
        context.db.delete_config(args.id).await?;
        println!(
            "{}",
            output::success(
                format!("Soft deleted config {}", args.id),
                output::color_enabled()
            )
        );
    }
    Ok(())
}

async fn delete_subscription(
    context: &AppContext,
    args: &DeleteSubscriptionArgs,
) -> crate::app::Result<()> {
    let subscription = context
        .db
        .get_subscription_by_id(args.id)
        .await?
        .ok_or_else(|| {
            crate::app::AppError::InvalidArgument(format!("subscription {} not found", args.id))
        })?;

    if !args.yes
        && !output::confirm(format!(
            "Delete subscription {} and its {} config(s)?",
            args.id, subscription.config_count
        ))?
    {
        println!("{}", output::notice("Aborted.", output::color_enabled()));
        return Ok(());
    }

    context.db.delete_subscription_with_configs(args.id).await?;
    println!(
        "{}",
        output::success(
            format!(
                "Deleted subscription {} and {} config(s)",
                args.id, subscription.config_count
            ),
            output::color_enabled()
        )
    );
    Ok(())
}

pub async fn restore(context: &AppContext, args: &RestoreArgs) -> crate::app::Result<()> {
    let config = context.db.get_config_by_id(args.id).await?.ok_or_else(|| {
        crate::app::AppError::InvalidArgument(format!("config {} not found", args.id))
    })?;

    if !config.is_deleted {
        println!(
            "{}",
            output::notice(
                format!("Config {} is not deleted", args.id),
                output::color_enabled()
            )
        );
        return Ok(());
    }

    context.db.restore_config(args.id).await?;
    println!(
        "{}",
        output::success(
            format!("Restored config {}", args.id),
            output::color_enabled()
        )
    );
    Ok(())
}

pub async fn show(context: &AppContext, args: &ShowArgs) -> crate::app::Result<()> {
    match &args.target {
        ShowTarget::Config(config_args) => show_config(context, config_args).await,
        ShowTarget::Subscription(subscription_args) => {
            show_subscription(context, subscription_args).await
        }
    }
}

async fn show_config(context: &AppContext, args: &ShowConfigArgs) -> crate::app::Result<()> {
    let config = context.db.get_config_by_id(args.id).await?.ok_or_else(|| {
        crate::app::AppError::InvalidArgument(format!("config {} not found", args.id))
    })?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "id": config.id,
                "subscription_id": config.subscription_id,
                "protocol": config.protocol,
                "address": config.address,
                "port": config.port,
                "username": config.username,
                "uuid": config.uuid,
                "password": config.password,
                "method": config.method,
                "network": config.network,
                "tls": config.tls,
                "sni": config.sni,
                "host": config.host,
                "path": config.path,
                "name": config.name,
                "is_active": config.is_active,
                "is_enabled": config.is_enabled,
                "is_deleted": config.is_deleted,
                "deleted_at": config.deleted_at,
                "imported_at": config.imported_at,
                "created_at": config.created_at,
                "updated_at": config.updated_at,
            }))?
        );
        return Ok(());
    }

    println!(
        "{}",
        output::format_kv(
            Some("Config"),
            &[
                ("id", config.id.to_string()),
                ("name", output::dash(config.name.as_deref())),
                ("protocol", config.protocol),
                ("address", format!("{}:{}", config.address, config.port)),
                ("network", config.network),
                ("tls", output::dash(config.tls.as_deref())),
                ("sni", output::dash(config.sni.as_deref())),
                ("host", output::dash(config.host.as_deref())),
                ("path", output::dash(config.path.as_deref())),
                ("active", output::bool_label(config.is_active).to_string()),
                ("enabled", output::bool_label(config.is_enabled).to_string()),
                ("deleted", output::bool_label(config.is_deleted).to_string()),
                ("deleted at", output::dash(config.deleted_at.as_deref())),
                ("imported at", config.imported_at),
                ("created at", config.created_at),
                ("updated at", config.updated_at),
            ],
            output::color_enabled(),
        )
    );
    Ok(())
}

async fn show_subscription(
    context: &AppContext,
    args: &ShowSubscriptionArgs,
) -> crate::app::Result<()> {
    let subscription = context
        .db
        .get_subscription_by_id(args.id)
        .await?
        .ok_or_else(|| {
            crate::app::AppError::InvalidArgument(format!("subscription {} not found", args.id))
        })?;

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&subscription_json(&subscription))?
        );
        return Ok(());
    }

    println!(
        "{}",
        output::format_kv(
            Some("Subscription"),
            &[
                ("id", subscription.id.to_string()),
                ("name", output::dash(subscription.name.as_deref())),
                ("kind", subscription.source_kind),
                ("source", output::dash(subscription.source_url.as_deref())),
                ("configs", subscription.config_count.to_string()),
                ("created at", subscription.created_at),
                ("updated at", subscription.updated_at),
            ],
            output::color_enabled(),
        )
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::config::AppConfig;
    use crate::app::context::RuntimePaths;
    use crate::db::{Database, DatabaseConnectionConfig};
    use crate::model::{Node, Protocol};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn enable_on_deleted_config_is_noop_ok() {
        let context = test_context("enable-deleted").await;
        let id = seed_config(&context).await;
        context.db.delete_config(id).await.expect("soft delete");

        enable(&context, &EnableArgs { id }).await.expect("noop ok");

        let config = fetch(&context, id).await;
        assert!(config.is_deleted);
    }

    #[tokio::test]
    async fn enable_already_enabled_is_noop_ok() {
        let context = test_context("enable-already").await;
        let id = seed_config(&context).await;
        assert!(fetch(&context, id).await.is_enabled);

        enable(&context, &EnableArgs { id }).await.expect("noop ok");
        assert!(fetch(&context, id).await.is_enabled);
    }

    #[tokio::test]
    async fn disable_then_enable_round_trip() {
        let context = test_context("enable-roundtrip").await;
        let id = seed_config(&context).await;

        disable(&context, &DisableArgs { id })
            .await
            .expect("disable ok");
        assert!(!fetch(&context, id).await.is_enabled);

        enable(&context, &EnableArgs { id })
            .await
            .expect("enable ok");
        assert!(fetch(&context, id).await.is_enabled);
    }

    #[tokio::test]
    async fn disable_already_disabled_is_noop_ok() {
        let context = test_context("disable-already").await;
        let id = seed_config(&context).await;
        disable(&context, &DisableArgs { id })
            .await
            .expect("disable ok");

        disable(&context, &DisableArgs { id })
            .await
            .expect("second disable noop ok");
        assert!(!fetch(&context, id).await.is_enabled);
    }

    async fn fetch(context: &AppContext, id: i64) -> crate::db::ConfigRecord {
        context
            .db
            .get_config_by_id(id)
            .await
            .expect("query should succeed")
            .expect("config should exist")
    }

    async fn seed_config(context: &AppContext) -> i64 {
        let source = crate::db::ImportSource {
            kind: crate::db::SourceKind::File,
            value: "seed.txt".to_string(),
            name: None,
        };
        context
            .db
            .import_nodes(&source, &[test_node("seed")])
            .await
            .expect("import should succeed");
        context
            .db
            .list_configs(&crate::db::ConfigListFilter::default())
            .await
            .expect("list should succeed")
            .into_iter()
            .next()
            .expect("config should exist")
            .id
    }

    fn test_node(name: &str) -> Node {
        Node {
            protocol: Protocol::Vless,
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: Some("uuid-123".to_string()),
            password: None,
            method: None,
            network: "ws".to_string(),
            tls: Some("tls".to_string()),
            sni: Some("cdn.example.com".to_string()),
            host: Some("cdn.example.com".to_string()),
            path: Some("/socket".to_string()),
            name: Some(name.to_string()),
            extensions: None,
            raw_config: format!("vless://uuid-123@example.com:443?type=ws#{name}"),
        }
    }

    async fn test_context(prefix: &str) -> AppContext {
        let root = std::env::temp_dir().join(format!(
            "xrat-lifecycle-{prefix}-{}",
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
