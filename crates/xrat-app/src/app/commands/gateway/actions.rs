use super::report::format_report;
use crate::app::{AppError, Result, config::AppConfig, services::gateway};
use crate::cli::{GatewayAction, GatewayArgs, GatewayInstallArgs};

pub(super) async fn run_with_engine_path(
    config: &AppConfig,
    args: &GatewayArgs,
    engine_path: Option<&std::path::Path>,
    runtime_dir: Option<&std::path::Path>,
) -> Result<()> {
    let (action, options, json) = match &args.action {
        GatewayAction::Doctor(args) => ("doctor", GatewayInstallArgs::default(), args.json),
        GatewayAction::Install(args) => (
            if args.dry_run {
                "install dry-run"
            } else {
                "install"
            },
            GatewayInstallArgs {
                interface: args.interface.clone(),
                lan_subnet: args.lan_subnet.clone(),
                local_subnets: args.local_subnets.clone(),
                dry_run: args.dry_run,
                json: args.json,
            },
            args.json,
        ),
        GatewayAction::Status(args) => ("status", GatewayInstallArgs::default(), args.json),
        GatewayAction::Remove(args) => (
            if args.dry_run {
                "remove dry-run"
            } else {
                "remove"
            },
            GatewayInstallArgs::default(),
            args.json,
        ),
    };
    let overrides = gateway::GatewayOverrides {
        interface: options.interface,
        lan_subnet: options.lan_subnet,
        local_subnets: options.local_subnets,
    };
    let mut settings = if matches!(args.action, GatewayAction::Remove(_)) {
        gateway::resolve(&Default::default(), &overrides)?
    } else {
        gateway::resolve(&config.gateway, &overrides)?
    };
    let preflight = if matches!(
        args.action,
        GatewayAction::Doctor(_) | GatewayAction::Install(_) | GatewayAction::Status(_)
    ) {
        Some(
            gateway::inspect(
                &mut settings,
                &config.runtime.engine,
                engine_path,
                runtime_dir,
                &config.runtime.tun.interface_name,
            )
            .await,
        )
    } else {
        None
    };
    let operation = match &args.action {
        GatewayAction::Doctor(_) | GatewayAction::Status(_) => Ok(
            "Gateway observations are read-only. Packet forwarding requires independent verification.",
        ),
        #[cfg(target_os = "linux")]
        GatewayAction::Install(install) => runtime_dir
            .ok_or_else(|| std::io::Error::other("gateway runtime directory is unavailable"))
            .and_then(|directory| {
                gateway::install(
                    directory,
                    &settings,
                    preflight.as_ref().expect("install preflight"),
                    install.dry_run,
                )
            }),
        #[cfg(target_os = "linux")]
        GatewayAction::Remove(remove) => runtime_dir
            .ok_or_else(|| std::io::Error::other("gateway runtime directory is unavailable"))
            .and_then(|directory| gateway::remove(directory, remove.dry_run)),
        #[cfg(not(target_os = "linux"))]
        GatewayAction::Install(_) | GatewayAction::Remove(_) => {
            Err(std::io::Error::other("gateway operations require Linux"))
        }
    };
    let observed = gateway::observe(runtime_dir);
    let message = match &operation {
        Ok(message) => (*message).to_string(),
        Err(error) => format!("Gateway operation failed: {error}"),
    };
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({"action": action, "settings": settings, "preflight": preflight, "installation": observed.installation, "protection": observed.protection, "packet_forwarding": "unverified", "diagnostic": message})
            )?
        );
    } else {
        println!(
            "{}",
            format_report(
                action,
                &settings,
                preflight.as_ref(),
                &format!(
                    "{message}\nInstallation: {} · Guard: {}",
                    observed.installation, observed.protection
                )
            )
        );
    }
    operation
        .map(|_| ())
        .map_err(|_| AppError::InvalidArgument(message))
}
