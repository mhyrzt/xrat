use super::entry::{subscription, unexpected};
use crate::app::Result;
use crate::app::commands::{lifecycle, output};
use crate::app::services::{
    DeleteOutcome, RestoreOutcome, ToggleOutcome,
    machine_service::{CatalogPayload, CatalogRequest, MachineClient},
};
use crate::cli::{DeleteArgs, DeleteTarget, ShowArgs, ShowTarget};
use xrat_model::ConfigRef;

pub(super) async fn toggle(client: &MachineClient, raw: &str, enabled: bool) -> Result<()> {
    let reference = ConfigRef::from(raw);
    let request = if enabled {
        CatalogRequest::Enable { reference }
    } else {
        CatalogRequest::Disable { reference }
    };
    let CatalogPayload::Toggle { config, outcome } = client.catalog(request).await? else {
        return Err(unexpected());
    };
    let (message, changed) = match outcome {
        ToggleOutcome::Changed => (
            format!(
                "{} config {}",
                if enabled { "Enabled" } else { "Disabled" },
                config.r#ref
            ),
            true,
        ),
        ToggleOutcome::AlreadyEnabled if enabled => {
            (format!("Config {raw} is already enabled"), false)
        }
        ToggleOutcome::AlreadyDisabled if !enabled => {
            (format!("Config {raw} is already disabled"), false)
        }
        ToggleOutcome::DeletedConfig => {
            (format!("Config {raw} is deleted; restore it first"), false)
        }
        _ => return Err(unexpected()),
    };
    println!(
        "{}",
        if changed {
            output::success(message, output::color_enabled())
        } else {
            output::notice(message, output::color_enabled())
        }
    );
    Ok(())
}

pub(super) async fn delete(client: &MachineClient, args: &DeleteArgs) -> Result<()> {
    match &args.target {
        DeleteTarget::Config(args) => {
            let request = CatalogRequest::Delete {
                reference: ConfigRef::from(args.id.as_str()),
                hard: args.hard,
            };
            let CatalogPayload::Delete { config, outcome } = client.catalog(request).await? else {
                return Err(unexpected());
            };
            let message = match outcome {
                DeleteOutcome::HardDeleted => {
                    format!("Permanently deleted config {}", config.r#ref)
                }
                DeleteOutcome::SoftDeleted => format!("Soft deleted config {}", config.r#ref),
                DeleteOutcome::AlreadyDeleted => {
                    println!(
                        "{}",
                        output::notice(
                            format!("Config {} is already deleted", args.id),
                            output::color_enabled()
                        )
                    );
                    return Ok(());
                }
            };
            println!("{}", output::success(message, output::color_enabled()));
        }
        DeleteTarget::Subscription(args) => {
            let record = subscription(client, &args.id).await?;
            if !args.yes
                && !output::confirm(format!(
                    "Delete subscription {} and its {} config(s)?",
                    record.r#ref, record.config_count
                ))?
            {
                println!("{}", output::notice("Aborted.", output::color_enabled()));
                return Ok(());
            }
            let CatalogPayload::Subscription(record) = client
                .catalog(CatalogRequest::DeleteSubscription {
                    reference: args.id.clone(),
                })
                .await?
            else {
                return Err(unexpected());
            };
            println!(
                "{}",
                output::success(
                    format!(
                        "Deleted subscription {} and {} config(s)",
                        record.r#ref, record.config_count
                    ),
                    output::color_enabled()
                )
            );
        }
    }
    Ok(())
}

pub(super) async fn restore(client: &MachineClient, raw: &str) -> Result<()> {
    let CatalogPayload::Restore { config, outcome } = client
        .catalog(CatalogRequest::Restore {
            reference: ConfigRef::from(raw),
        })
        .await?
    else {
        return Err(unexpected());
    };
    println!(
        "{}",
        match outcome {
            RestoreOutcome::Restored => output::success(
                format!("Restored config {}", config.r#ref),
                output::color_enabled()
            ),
            RestoreOutcome::NotDeleted => output::notice(
                format!("Config {raw} is not deleted"),
                output::color_enabled()
            ),
        }
    );
    Ok(())
}

pub(super) async fn show(client: &MachineClient, args: &ShowArgs) -> Result<()> {
    match &args.target {
        ShowTarget::Config(args) => {
            let config = client.config(&ConfigRef::from(args.id.as_str())).await?;
            if args.raw {
                return lifecycle::print_raw_contents(
                    client.raw_configs_by_ids(&[config.id]).await?,
                );
            }
            let subscription_ref = match config.subscription_id {
                Some(id) => Some(subscription(client, &id.to_string()).await?.r#ref),
                None => None,
            };
            lifecycle::print_config(config, subscription_ref, args.json)
        }
        ShowTarget::Subscription(args) => {
            lifecycle::print_subscription(subscription(client, &args.id).await?, args.json)
        }
    }
}

pub(super) async fn purge(client: &MachineClient, args: &crate::cli::PurgeArgs) -> Result<()> {
    let pending = client
        .list(&crate::app::services::ConfigListRequest {
            only_deleted: true,
            offset: Some(0),
            limit: Some(0),
            ..Default::default()
        })
        .await?
        .total;
    if !crate::app::commands::purge::confirm(pending, args.yes)? {
        return Ok(());
    }
    let CatalogPayload::Changed(count) = client.catalog(CatalogRequest::Purge {}).await? else {
        return Err(unexpected());
    };
    crate::app::commands::purge::print_count(count);
    Ok(())
}
