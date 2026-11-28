use super::super::output::{self, Align, Cell, Column};
use crate::app::services::gateway;
use std::io::IsTerminal;

pub(super) fn format_report(
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
