use super::output::{self, Align, Cell, Column};
use crate::app::{
    AppError, Result,
    config::{self, AppConfig},
    services::gateway,
};
use crate::cli::{Cli, GatewayAction, GatewayArgs, GatewayInstallArgs};
use std::io::IsTerminal;

pub async fn run(cli: &Cli, args: &GatewayArgs) -> Result<()> {
    let path = match &cli.config {
        Some(path) => path.clone(),
        None => crate::app::app_paths::resolve()?.config_path,
    };
    let config = if path.exists() {
        config::load(&path)?
    } else if cli.config.is_some() {
        return Err(AppError::InvalidArgument(format!(
            "configuration file does not exist: {}",
            path.display()
        )));
    } else {
        AppConfig::default()
    };
    let engine_path = match config.runtime.engine.as_str() {
        "xray" => cli
            .xray
            .clone()
            .or_else(|| {
                config
                    .paths
                    .xray
                    .as_ref()
                    .map(|value| config::resolve_config_path(&path, value))
            })
            .unwrap_or_else(|| "xray".into()),
        "sing-box" => cli
            .sing_box
            .clone()
            .or_else(|| {
                config
                    .paths
                    .sing_box
                    .as_ref()
                    .map(|value| config::resolve_config_path(&path, value))
            })
            .unwrap_or_else(|| "sing-box".into()),
        _ => std::path::PathBuf::new(),
    };
    let runtime_dir = crate::app::app_paths::resolve()
        .ok()
        .map(|paths| paths.root_dir.join("runtime"));
    run_with_engine_path(&config, args, Some(&engine_path), runtime_dir.as_deref()).await
}

pub async fn run_with_config(
    config: &AppConfig,
    args: &GatewayArgs,
    runtime_dir: &std::path::Path,
) -> Result<()> {
    run_with_engine_path(config, args, None, Some(runtime_dir)).await
}

async fn run_with_engine_path(
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
    let mut settings = gateway::resolve(&config.gateway, &overrides)?;
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

fn format_report(
    action: &str,
    settings: &gateway::EffectiveSettings,
    report: Option<&gateway::Report>,
    message: &str,
) -> String {
    let color = output::color_enabled();
    let glyphs = std::io::stdout().is_terminal();
    let styled = |text: &str, style| output::style_text(text, style, color);
    let title = |text: &str| styled(text, output::Style::Cyan);
    let value = |v: &gateway::ValueWithSource<Option<String>>| match &v.value {
        Some(value) => format!("{value} ({})", v.source),
        None => "not determined".to_string(),
    };
    let local = if settings.local_subnets.value.is_empty() {
        "none".to_string()
    } else {
        format!(
            "{} ({})",
            settings.local_subnets.value.join(", "),
            settings.local_subnets.source
        )
    };
    let rows = [
        ("LAN interface", value(&settings.interface)),
        ("LAN subnet", value(&settings.lan_subnet)),
        ("Extra local subnets", local),
    ];
    let mut lines = vec![
        title(&format!(
            "{} XRAT Gateway · {action}",
            if glyphs { "◆" } else { "*" }
        )),
        String::new(),
        title(if glyphs {
            "▸ Configuration"
        } else {
            "Configuration"
        }),
        output::format_kv(None, &rows, color),
    ];
    if let Some(report) = report {
        if !report.interfaces.is_empty() {
            lines.push(String::new());
            lines.push(title(if glyphs {
                "▸ IPv4 interfaces"
            } else {
                "IPv4 interfaces"
            }));
            let columns = ["", "Interface", "Address", "Subnet", "Status"].map(|header| Column {
                header,
                align: Align::Left,
            });
            let rows = report
                .interfaces
                .iter()
                .map(|iface| {
                    let marker = if iface.candidate {
                        (if glyphs { "●" } else { "+" }, output::Style::Cyan)
                    } else {
                        (if glyphs { "○" } else { "-" }, output::Style::Dim)
                    };
                    vec![
                        Cell::styled(marker.0, marker.1),
                        Cell::plain(&iface.name),
                        Cell::plain(iface.address.to_string()),
                        Cell::plain(&iface.subnet),
                        Cell::plain(if iface.candidate {
                            "LAN candidate"
                        } else if !iface.up {
                            "down"
                        } else if iface.point_to_point {
                            "point-to-point"
                        } else {
                            "not a LAN candidate"
                        }),
                    ]
                })
                .collect::<Vec<_>>();
            lines.push(output::format_table(&columns, &rows, color));
        }
        if !report.default_routes.is_empty() {
            lines.push(String::new());
            lines.push(title(if glyphs {
                "▸ Main-table default routes"
            } else {
                "Main-table default routes"
            }));
            let columns = ["", "Interface", "Gateway", "Metric"].map(|header| Column {
                header,
                align: Align::Left,
            });
            let rows = report
                .default_routes
                .iter()
                .map(|route| {
                    vec![
                        Cell::styled(if glyphs { "↗" } else { ">" }, output::Style::Cyan),
                        Cell::plain(&route.interface),
                        Cell::plain(route.gateway.to_string()),
                        Cell::plain(route.metric.to_string()),
                    ]
                })
                .collect::<Vec<_>>();
            lines.push(output::format_table(&columns, &rows, color));
        }
        if !report.policy_rules.is_empty() {
            lines.push(String::new());
            lines.push(title(if glyphs {
                "▸ IPv4 policy rules"
            } else {
                "IPv4 policy rules"
            }));
            let columns = [
                "Priority",
                "Table",
                "Action",
                "Source prefix",
                "Destination prefix",
            ]
            .map(|header| Column {
                header,
                align: Align::Left,
            });
            let rows = report
                .policy_rules
                .iter()
                .map(|rule| {
                    vec![
                        Cell::plain(
                            rule.priority()
                                .map(|value| value.to_string())
                                .unwrap_or_else(|| "unknown".into()),
                        ),
                        Cell::plain(
                            rule.attributes
                                .iter()
                                .find(|attribute| attribute.kind == 15)
                                .and_then(|attribute| {
                                    <[u8; 4]>::try_from(attribute.value.as_slice()).ok()
                                })
                                .map(u32::from_ne_bytes)
                                .unwrap_or(rule.table as u32)
                                .to_string(),
                        ),
                        Cell::plain(rule.action.to_string()),
                        Cell::plain(rule.source_prefix.to_string()),
                        Cell::plain(rule.destination_prefix.to_string()),
                    ]
                })
                .collect::<Vec<_>>();
            lines.push(output::format_table(&columns, &rows, color));
            lines.push("  Selectors and referenced tables require further inspection.".into());
        }
        lines.push(String::new());
        lines.push(title(if glyphs { "▸ Checks" } else { "Checks" }));
        for check in &report.checks {
            let label = match check.name {
                "lan_interface" => "LAN interface",
                "lan_subnet" => "LAN subnet",
                "upstream_routes" => "Upstream routes",
                "upstream_topology" => "Upstream topology",
                "caller_privileges" => "Caller privileges",
                "tun_device" => "TUN device",
                "remaining_checks" => "Remaining checks",
                "policy_rules" => "Policy rules",
                "routing_tables" => "Routing tables",
                "runtime_owner" => "Runtime owner",
                "firewall_tool" => "Firewall tool",
                "engine" => "Engine preference",
                "firewall_chains" => "Firewall chains",
                "rule_marks" => "Policy marks",
                "tun_collision" => "TUN ownership",
                "network_sysctls" => "IPv4 settings",
                "recovery" => "Recovery",
                "interfaces" => "Interfaces",
                "platform" => "Platform",
                other => other,
            };
            let (symbol, state, style) = match check.state {
                "ok" => (if glyphs { "✓" } else { "+" }, "OK", output::Style::Green),
                "failed" => (if glyphs { "✗" } else { "!" }, "FAIL", output::Style::Red),
                "unknown" => ("?", "UNKNOWN", output::Style::Dim),
                _ => (
                    if glyphs { "◐" } else { "~" },
                    "INCOMPLETE",
                    output::Style::Yellow,
                ),
            };
            lines.push(format!(
                "  {} {}  {}",
                styled(&format!("{symbol} {state:<10}"), style),
                styled(&format!("{label:<17}"), output::Style::Cyan),
                check.detail
            ));
        }
    }
    lines.push(String::new());
    lines.push(format!(
        "{} {message}",
        styled(if glyphs { "⚠" } else { "!" }, output::Style::Yellow)
    ));
    lines.join("\n")
}
