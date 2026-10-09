use super::prelude::*;
use crate::app::config::TunSplitMode;
use crate::app::services::split_tunnel;
use crate::tui::app::{SplitListTab, SplitModalMode, SplitPane};
use crate::tui::theme;

pub fn render_split_modal(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let Some(modal) = &app.split_modal else {
        return;
    };
    let width = area.width.saturating_sub(4).clamp(56, 112);
    let height = area.height.saturating_sub(2).clamp(18, 36);
    let modal_area = centered_rect_fixed(width, height, area);
    frame.render_widget(Clear, modal_area);

    let dirty = if modal.is_dirty() { " *" } else { "" };
    let title = format!(" TUN Split Tunneling{dirty} ");
    let footer = split_footer(modal.mode(), modal.pane);
    let outer = Block::default()
        .title(Line::styled(title, theme::accent_style().bold()))
        .title_bottom(footer)
        .borders(Borders::ALL)
        .border_style(theme::muted_style())
        .padding(Padding::horizontal(1));
    let inner = outer.inner(modal_area);
    frame.render_widget(outer, modal_area);

    let has_banner = modal.error.is_some() || modal.notice.is_some();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(if has_banner { 2 } else { 0 }),
        ])
        .split(inner);

    let tun_badge = if modal.tun_enabled {
        Span::styled(" ON ", theme::success_style().bold())
    } else {
        Span::styled(" OFF ", theme::muted_style().bold())
    };
    let mode_spans = [
        ("all", modal.split_mode == TunSplitMode::All),
        ("blacklist", modal.split_mode == TunSplitMode::Blacklist),
        ("whitelist", modal.split_mode == TunSplitMode::Whitelist),
    ];
    let mut header_spans = vec![
        Span::styled("TUN (u): ", theme::muted_style()),
        tun_badge,
        Span::raw("   "),
        Span::styled("Mode (m): ", theme::muted_style()),
    ];
    for (idx, (label, active)) in mode_spans.into_iter().enumerate() {
        if idx > 0 {
            header_spans.push(Span::styled(" | ", theme::muted_style()));
        }
        header_spans.push(if active {
            Span::styled(format!("[{label}]"), theme::accent_style().bold())
        } else {
            Span::styled(label, theme::chrome_style())
        });
    }
    header_spans.push(Span::raw("   "));
    header_spans.push(Span::styled("List (b/w): ", theme::muted_style()));
    let bl_active = modal.list_tab == SplitListTab::Blacklist;
    header_spans.push(if bl_active {
        Span::styled(
            format!("[blacklist ({})]", modal.blacklist.len()),
            theme::accent_style().bold(),
        )
    } else {
        Span::styled(
            format!("blacklist ({})", modal.blacklist.len()),
            theme::chrome_style(),
        )
    });
    header_spans.push(Span::styled(" | ", theme::muted_style()));
    header_spans.push(if !bl_active {
        Span::styled(
            format!("[whitelist ({})]", modal.whitelist.len()),
            theme::accent_style().bold(),
        )
    } else {
        Span::styled(
            format!("whitelist ({})", modal.whitelist.len()),
            theme::chrome_style(),
        )
    });

    let mode_hint = match modal.split_mode {
        TunSplitMode::All => "All TUN traffic routes through the VPN.",
        TunSplitMode::Blacklist => {
            "All TUN traffic routes through the VPN except applications in blacklist (routed direct)."
        }
        TunSplitMode::Whitelist => {
            "Only applications in whitelist route through the VPN; all other TUN traffic routes direct."
        }
    };
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(header_spans),
            Line::styled(mode_hint, theme::muted_style()),
        ]),
        rows[0],
    );

    let (input_title, input_text, input_active) = if let Some(adding) = &modal.adding_input {
        (
            format!(
                " Add to {} (binary, /abs/path, /abs/dir/, or .desktop) ",
                modal.list_tab.as_str()
            ),
            format!("{adding}█"),
            true,
        )
    } else if modal.searching {
        (
            " Filter Discovered Apps ".to_string(),
            format!("{}█", modal.search_query),
            true,
        )
    } else {
        (
            " Input ".to_string(),
            if modal.search_query.is_empty() {
                "Press `a` to add custom app/path/dir, or `/` to filter discovered apps".to_string()
            } else {
                format!("Filter: {}", modal.search_query)
            },
            false,
        )
    };
    frame.render_widget(
        Paragraph::new(input_text)
            .style(if input_active {
                theme::accent_style()
            } else {
                theme::muted_style()
            })
            .block(
                Block::default()
                    .title(input_title)
                    .borders(Borders::ALL)
                    .border_style(if input_active {
                        theme::accent_style()
                    } else {
                        theme::muted_style()
                    }),
            ),
        rows[1],
    );

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(rows[2]);

    let active_list = modal.active_list();
    let cfg_rows = columns[0].height.saturating_sub(2) as usize;
    let cfg_start = modal
        .configured_index
        .saturating_sub(cfg_rows.saturating_sub(1));
    let desktop_dirs = split_tunnel::desktop_search_dirs();
    let cfg_lines: Vec<Line> = if active_list.is_empty() {
        vec![Line::styled(
            "  (empty — select apps on the right or press `a`)",
            theme::muted_style(),
        )]
    } else {
        active_list
            .iter()
            .enumerate()
            .skip(cfg_start)
            .take(cfg_rows)
            .map(|(index, spec)| {
                let selected = index == modal.configured_index;
                let marker = if selected { "› " } else { "  " };
                let resolved = split_tunnel::normalize_app_specifier_with_dirs(spec, &desktop_dirs)
                    .ok()
                    .filter(|r| r.len() != 1 || r[0] != *spec)
                    .map(|r| format!(" → {}", r.join(", ")))
                    .unwrap_or_default();
                Line::from(vec![
                    Span::styled(
                        format!("{marker}{spec}"),
                        if selected {
                            theme::accent_style().bold()
                        } else {
                            theme::chrome_style()
                        },
                    ),
                    Span::styled(resolved, theme::muted_style()),
                ])
            })
            .collect()
    };
    frame.render_widget(
        Paragraph::new(cfg_lines).block(
            Block::default()
                .title(format!(
                    " {} ({}) ",
                    modal.list_tab.as_str(),
                    active_list.len()
                ))
                .borders(Borders::ALL)
                .border_style(if modal.pane == SplitPane::Configured {
                    theme::accent_style()
                } else {
                    theme::muted_style()
                }),
        ),
        columns[0],
    );

    let visible_disc = modal.visible_discovered_indices();
    let disc_rows = columns[1].height.saturating_sub(2) as usize;
    let disc_start = modal
        .discovered_index
        .saturating_sub(disc_rows.saturating_sub(1));
    let disc_lines: Vec<Line> = if visible_disc.is_empty() {
        vec![Line::styled(
            "  No matching applications found.",
            theme::muted_style(),
        )]
    } else {
        visible_disc
            .iter()
            .enumerate()
            .skip(disc_start)
            .take(disc_rows)
            .map(|(pos, &app_idx)| {
                let app_entry = &modal.discovered_apps[app_idx];
                let selected = pos == modal.discovered_index;
                let in_list = active_list.iter().any(|item| item == &app_entry.rule_entry);
                let marker = if selected { "›" } else { " " };
                let check = if in_list { "✓" } else { " " };
                let source_badge = match app_entry.source {
                    split_tunnel::AppSource::Running => "[run]",
                    split_tunnel::AppSource::Desktop => "[app]",
                    split_tunnel::AppSource::RunningAndDesktop => "[run+app]",
                };
                Line::from(vec![
                    Span::styled(
                        format!("{marker}{check} {:<16}", app_entry.rule_entry),
                        if selected {
                            theme::accent_style().bold()
                        } else if in_list {
                            theme::success_style()
                        } else {
                            theme::chrome_style()
                        },
                    ),
                    Span::styled(format!(" {:<9} ", source_badge), theme::muted_style()),
                    Span::styled(app_entry.name.clone(), theme::muted_style()),
                ])
            })
            .collect()
    };
    frame.render_widget(
        Paragraph::new(disc_lines).block(
            Block::default()
                .title(format!(
                    " Discovered Apps · {}/{} ",
                    modal
                        .discovered_index
                        .saturating_add(1)
                        .min(visible_disc.len()),
                    visible_disc.len()
                ))
                .borders(Borders::ALL)
                .border_style(if modal.pane == SplitPane::Discovered {
                    theme::accent_style()
                } else {
                    theme::muted_style()
                }),
        ),
        columns[1],
    );

    if has_banner {
        let banner_line = if let Some(error) = &modal.error {
            Line::styled(error.clone(), theme::failure_style().bold())
        } else if let Some(notice) = &modal.notice {
            Line::styled(notice.clone(), theme::success_style().bold())
        } else {
            Line::raw("")
        };
        frame.render_widget(Paragraph::new(banner_line), rows[3]);
    }
}

fn split_footer(mode: SplitModalMode, pane: SplitPane) -> Line<'static> {
    let pairs: &[(&str, &str)] = match mode {
        SplitModalMode::AddInput | SplitModalMode::Search => {
            &[("↵", "confirm"), ("⌃U", "clear"), ("Esc", "cancel")]
        }
        SplitModalMode::Browse => match pane {
            SplitPane::Configured => &[
                ("⇥", "apps"),
                ("m", "mode"),
                ("b/w", "list"),
                ("a", "add"),
                ("d", "remove"),
                ("c", "clear"),
                ("⌃S", "save & apply"),
                ("Esc", "close"),
            ],
            SplitPane::Discovered => &[
                ("⇥", "list"),
                ("␣/↵", "toggle app"),
                ("/", "filter"),
                ("r", "refresh"),
                ("m", "mode"),
                ("b/w", "list"),
                ("⌃S", "save & apply"),
                ("Esc", "close"),
            ],
        },
    };
    let mut spans = vec![Span::raw(" ")];
    for (idx, (key, label)) in pairs.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::styled(" · ", theme::muted_style()));
        }
        spans.push(Span::styled(
            (*key).to_string(),
            theme::accent_style().bold(),
        ));
        spans.push(Span::styled(format!(" {label}"), theme::chrome_style()));
    }
    spans.push(Span::raw(" "));
    Line::from(spans)
}
