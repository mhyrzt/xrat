use crate::app::commands::list::subscription_json;
use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::app::services::{DeleteOutcome, RestoreOutcome, ToggleOutcome};
use crate::cli::{
    DeleteArgs, DeleteConfigArgs, DeleteSubscriptionArgs, DeleteTarget, DisableArgs, EnableArgs,
    RestoreArgs, ShowArgs, ShowConfigArgs, ShowSubscriptionArgs, ShowTarget,
};

pub async fn enable(context: &AppContext, args: &EnableArgs) -> crate::app::Result<()> {
    let lifecycle = context.services().lifecycle;
    match lifecycle
        .enable(&xrat_model::ConfigRef::from(args.id.as_str()))
        .await?
    {
        ToggleOutcome::Changed => {
            let config = lifecycle
                .config(&xrat_model::ConfigRef::from(args.id.as_str()))
                .await?;
            println!(
                "{}",
                output::success(
                    format!("Enabled config {}", config.r#ref),
                    output::color_enabled()
                )
            );
        }
        ToggleOutcome::AlreadyEnabled => {
            print_notice(format!("Config {} is already enabled", args.id))
        }
        ToggleOutcome::AlreadyDisabled => unreachable!("enable never reports AlreadyDisabled"),
        ToggleOutcome::DeletedConfig => {
            print_notice(format!("Config {} is deleted; restore it first", args.id))
        }
    }
    Ok(())
}

pub async fn disable(context: &AppContext, args: &DisableArgs) -> crate::app::Result<()> {
    let lifecycle = context.services().lifecycle;
    match lifecycle
        .disable(&xrat_model::ConfigRef::from(args.id.as_str()))
        .await?
    {
        ToggleOutcome::Changed => {
            let config = lifecycle
                .config(&xrat_model::ConfigRef::from(args.id.as_str()))
                .await?;
            println!(
                "{}",
                output::success(
                    format!("Disabled config {}", config.r#ref),
                    output::color_enabled()
                )
            );
        }
        ToggleOutcome::AlreadyDisabled => {
            print_notice(format!("Config {} is already disabled", args.id))
        }
        ToggleOutcome::AlreadyEnabled => {
            unreachable!("disable never reports AlreadyEnabled")
        }
        ToggleOutcome::DeletedConfig => {
            print_notice(format!("Config {} is deleted; restore it first", args.id))
        }
    }
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
    let lifecycle = context.services().lifecycle;
    let (config, outcome) = lifecycle
        .delete(&xrat_model::ConfigRef::from(args.id.as_str()), args.hard)
        .await?;
    match outcome {
        DeleteOutcome::HardDeleted => println!(
            "{}",
            output::success(
                format!("Permanently deleted config {}", config.r#ref),
                output::color_enabled()
            )
        ),
        DeleteOutcome::SoftDeleted => println!(
            "{}",
            output::success(
                format!("Soft deleted config {}", config.r#ref),
                output::color_enabled()
            )
        ),
        DeleteOutcome::AlreadyDeleted => {
            print_notice(format!("Config {} is already deleted", args.id))
        }
    }
    Ok(())
}

async fn delete_subscription(
    context: &AppContext,
    args: &DeleteSubscriptionArgs,
) -> crate::app::Result<()> {
    let lifecycle = context.services().lifecycle;
    let subscription = lifecycle.subscription(&args.id).await?;

    if !args.yes
        && !output::confirm(format!(
            "Delete subscription {} and its {} config(s)?",
            subscription.r#ref, subscription.config_count
        ))?
    {
        println!("{}", output::notice("Aborted.", output::color_enabled()));
        return Ok(());
    }

    let subscription = lifecycle.delete_subscription(&args.id).await?;
    println!(
        "{}",
        output::success(
            format!(
                "Deleted subscription {} and {} config(s)",
                subscription.r#ref, subscription.config_count
            ),
            output::color_enabled()
        )
    );
    Ok(())
}

pub async fn restore(context: &AppContext, args: &RestoreArgs) -> crate::app::Result<()> {
    let lifecycle = context.services().lifecycle;
    let (config, outcome) = lifecycle
        .restore(&xrat_model::ConfigRef::from(args.id.as_str()))
        .await?;
    match outcome {
        RestoreOutcome::Restored => println!(
            "{}",
            output::success(
                format!("Restored config {}", config.r#ref),
                output::color_enabled()
            )
        ),
        RestoreOutcome::NotDeleted => print_notice(format!("Config {} is not deleted", args.id)),
    }
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

fn print_notice(message: String) {
    println!("{}", output::notice(message, output::color_enabled()));
}

async fn show_config(context: &AppContext, args: &ShowConfigArgs) -> crate::app::Result<()> {
    let config = context
        .services()
        .lifecycle
        .config(&xrat_model::ConfigRef::from(args.id.as_str()))
        .await?;

    if args.raw {
        return print_raw_contents(
            context
                .services()
                .configs
                .raw_configs_by_ids(&[config.id])
                .await?,
        );
    }

    let subscription_ref = match config.subscription_id {
        Some(subscription_id) => context
            .db
            .get_subscription_by_id(subscription_id)
            .await?
            .map(|subscription| subscription.r#ref),
        None => None,
    };
    print_config(config, subscription_ref, args.json)
}

pub(super) fn print_raw_contents(contents: Option<Vec<String>>) -> crate::app::Result<()> {
    use std::io::Write;
    let raw = contents
        .and_then(|mut values| values.pop())
        .ok_or_else(|| crate::app::AppError::InvalidArgument("config was not found".into()))?;
    std::io::stdout().lock().write_all(raw.as_bytes())?;
    Ok(())
}

pub(super) fn print_config(
    config: xrat_db::ConfigRecord,
    subscription_ref: Option<String>,
    json: bool,
) -> crate::app::Result<()> {
    let is_xray_profile = config.is_xray_profile();
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ref": config.r#ref,
                "subscription_ref": subscription_ref,
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

    let subscription_ref = subscription_ref.unwrap_or_else(|| "-".into());

    println!(
        "{}",
        output::format_kv(
            Some("Config"),
            &[
                ("ref", config.r#ref),
                ("subscription", subscription_ref),
                ("name", output::dash(config.name.as_deref())),
                ("protocol", config.protocol),
                (
                    "address",
                    if is_xray_profile {
                        "complete Xray profile".into()
                    } else {
                        format!("{}:{}", config.address, config.port)
                    }
                ),
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
    let subscription = context.services().lifecycle.subscription(&args.id).await?;

    print_subscription(subscription, args.json)
}

pub(super) fn print_subscription(
    subscription: xrat_db::SubscriptionRecord,
    json: bool,
) -> crate::app::Result<()> {
    if json {
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
                ("ref", subscription.r#ref),
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
mod tests;
