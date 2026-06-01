use crate::app::context::AppContext;
use crate::cli::{DeleteArgs, DisableArgs, EnableArgs, RestoreArgs, SelectArgs, ShowArgs};

pub async fn select(context: &AppContext, args: &SelectArgs) -> crate::app::Result<()> {
    let config = context.db.get_config_by_id(args.id).await?.ok_or_else(|| {
        crate::app::AppError::InvalidArgument(format!("config {} not found", args.id))
    })?;

    if config.is_deleted {
        return Err(crate::app::AppError::InvalidArgument(format!(
            "config {} is deleted",
            args.id
        )));
    }

    context.db.set_selected_config(args.id).await?;
    println!("Selected config {}", args.id);
    Ok(())
}

pub async fn enable(context: &AppContext, args: &EnableArgs) -> crate::app::Result<()> {
    let config = context.db.get_config_by_id(args.id).await?.ok_or_else(|| {
        crate::app::AppError::InvalidArgument(format!("config {} not found", args.id))
    })?;

    if config.is_deleted {
        return Err(crate::app::AppError::InvalidArgument(format!(
            "config {} is deleted",
            args.id
        )));
    }

    context.db.set_config_enabled(args.id, true).await?;
    println!("Enabled config {}", args.id);
    Ok(())
}

pub async fn disable(context: &AppContext, args: &DisableArgs) -> crate::app::Result<()> {
    let config = context.db.get_config_by_id(args.id).await?.ok_or_else(|| {
        crate::app::AppError::InvalidArgument(format!("config {} not found", args.id))
    })?;

    if config.is_deleted {
        return Err(crate::app::AppError::InvalidArgument(format!(
            "config {} is deleted",
            args.id
        )));
    }

    context.db.set_config_enabled(args.id, false).await?;
    println!("Disabled config {}", args.id);
    Ok(())
}

pub async fn delete(context: &AppContext, args: &DeleteArgs) -> crate::app::Result<()> {
    let config = context.db.get_config_by_id(args.id).await?.ok_or_else(|| {
        crate::app::AppError::InvalidArgument(format!("config {} not found", args.id))
    })?;

    if args.hard {
        context.db.hard_delete_config(args.id).await?;
        println!("Permanently deleted config {}", args.id);
    } else {
        if config.is_deleted {
            println!("Config {} is already deleted", args.id);
            return Ok(());
        }
        context.db.delete_config(args.id).await?;
        println!("Soft deleted config {}", args.id);
    }
    Ok(())
}

pub async fn restore(context: &AppContext, args: &RestoreArgs) -> crate::app::Result<()> {
    let config = context.db.get_config_by_id(args.id).await?.ok_or_else(|| {
        crate::app::AppError::InvalidArgument(format!("config {} not found", args.id))
    })?;

    if !config.is_deleted {
        println!("Config {} is not deleted", args.id);
        return Ok(());
    }

    context.db.restore_config(args.id).await?;
    println!("Restored config {}", args.id);
    Ok(())
}

pub async fn show(context: &AppContext, args: &ShowArgs) -> crate::app::Result<()> {
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
                "is_selected": config.is_selected,
                "is_deleted": config.is_deleted,
                "deleted_at": config.deleted_at,
                "imported_at": config.imported_at,
                "created_at": config.created_at,
                "updated_at": config.updated_at,
            }))?
        );
        return Ok(());
    }

    println!("ID:           {}", config.id);
    println!("Name:         {}", config.name.as_deref().unwrap_or("-"));
    println!("Protocol:     {}", config.protocol);
    println!("Address:      {}:{}", config.address, config.port);
    println!("Network:      {}", config.network);
    println!("TLS:          {}", config.tls.as_deref().unwrap_or("-"));
    println!("SNI:          {}", config.sni.as_deref().unwrap_or("-"));
    println!("Host:         {}", config.host.as_deref().unwrap_or("-"));
    println!("Path:         {}", config.path.as_deref().unwrap_or("-"));
    println!("Active:       {}", config.is_active);
    println!("Enabled:      {}", config.is_enabled);
    println!("Selected:     {}", config.is_selected);
    println!("Deleted:      {}", config.is_deleted);
    if let Some(deleted_at) = &config.deleted_at {
        println!("Deleted at:   {}", deleted_at);
    }
    println!("Imported at:  {}", config.imported_at);
    println!("Created at:   {}", config.created_at);
    println!("Updated at:   {}", config.updated_at);
    Ok(())
}
