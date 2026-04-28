use crate::app::runtime::AppContext;
use crate::cli::{ListArgs, ListConfigsArgs, ListSubscriptionsArgs, ListTarget};
use crate::db::ConfigListFilter;

pub async fn run(context: &AppContext, command: &ListArgs) -> crate::app::Result<()> {
    match &command.target {
        ListTarget::Configs(filters) => print_configs(context, filters).await?,
        ListTarget::Subscriptions(filters) => print_subscriptions(context, filters).await?,
    }

    Ok(())
}

async fn print_configs(context: &AppContext, filters: &ListConfigsArgs) -> crate::app::Result<()> {
    let filter = build_config_list_filter(filters);
    let configs = context.db.list_configs(&filter).await?;

    if configs.is_empty() {
        println!("No configs found for the requested filters.");
        return Ok(());
    }

    println!("ID\tSUB\tPROTO\tADDRESS\tPORT\tNAME\tFLAGS");
    for config in configs {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            config.id,
            config
                .subscription_id
                .map_or("-".to_string(), |id| id.to_string()),
            config.protocol,
            config.address,
            config.port,
            config.name.unwrap_or_else(|| "-".to_string()),
            format_config_flags(config.is_enabled, config.is_selected, config.is_active),
        );
    }

    Ok(())
}

async fn print_subscriptions(
    context: &AppContext,
    filters: &ListSubscriptionsArgs,
) -> crate::app::Result<()> {
    let mut subscriptions = context.db.list_subscriptions().await?;
    if let Some(kind) = &filters.kind {
        subscriptions.retain(|subscription| subscription.source_kind == kind.as_str());
    }

    if subscriptions.is_empty() {
        println!("No subscriptions found for the requested filters.");
        return Ok(());
    }

    println!("ID\tKIND\tCONFIGS\tNAME\tSOURCE");
    for subscription in subscriptions {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            subscription.id,
            subscription.source_kind,
            subscription.config_count,
            subscription.name.unwrap_or_else(|| "-".to_string()),
            subscription.source_url.unwrap_or_else(|| "-".to_string()),
        );
    }

    Ok(())
}

fn format_config_flags(is_enabled: bool, is_selected: bool, is_active: bool) -> String {
    let mut flags = Vec::new();

    if is_enabled {
        flags.push("enabled");
    } else {
        flags.push("disabled");
    }
    if is_selected {
        flags.push("selected");
    }
    if is_active {
        flags.push("active");
    }

    flags.join(",")
}

fn build_config_list_filter(args: &ListConfigsArgs) -> ConfigListFilter {
    ConfigListFilter {
        only_enabled: args.enabled_only,
        only_selected: args.selected_only,
        only_active: args.active_only,
        subscription_id: args.subscription,
    }
}
