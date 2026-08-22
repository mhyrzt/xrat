use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use crate::app::config::{EditableSetting, SettingKind, SettingValue};
use crate::tui::app::{ImportModalStep, SettingsMode, SettingsPane, TuiApp};
use crate::tui::theme;

pub fn render_help(frame: &mut Frame<'_>, area: Rect) {
    // Sections arranged in a 4-row x 3-column grid. Each row is padded so the
    // section headers line up horizontally across all three columns.
    let section = |title: &'static str, mut rows: Vec<Line<'static>>| -> Vec<Line<'static>> {
        let mut lines = vec![Line::styled(title, theme::muted_style())];
        lines.append(&mut rows);
        lines
    };

    let grid: [[Vec<Line>; 3]; 4] = [
        [
            section(
                "Navigation",
                vec![
                    help_line("[ / ]", "Previous / Next tab"),
                    help_line("⇥ / ⇤", "Focus next / prev card"),
                    help_line("j, ↓ / k, ↑", "Scroll down/up"),
                    help_line("PgUp / PgDn", "Page up/down"),
                    help_line("Home / End", "Jump top/bottom"),
                    help_line("i", "Import link"),
                    help_line(",", "Settings"),
                    help_line("Esc", "Close modal / back"),
                    help_line("q", "Quit"),
                ],
            ),
            section(
                "Configs",
                vec![
                    help_line("↵", "Start focused"),
                    help_line("e", "Enable focused"),
                    help_line("x", "Disable focused"),
                    help_line("y", "Show QR"),
                    help_line("c", "Copy link"),
                ],
            ),
            section(
                "Tests",
                vec![
                    help_line("t t", "Focused"),
                    help_line("t a", "All enabled"),
                    help_line("t v", "Visible"),
                    help_line("t r", "Failed"),
                    help_line("t s", "Stale"),
                    help_line("t c", "Cancel batch"),
                ],
            ),
        ],
        [
            section(
                "Search",
                vec![
                    help_line("/", "Search configs"),
                    help_line("Esc", "Cancel search"),
                    help_line("⌃U", "Clear search input"),
                ],
            ),
            section(
                "Runtime",
                vec![help_line("K", "Kill"), help_line("R", "Restart")],
            ),
            section(
                "Soft Delete",
                vec![
                    help_line("d d", "Focused"),
                    help_line("d f", "All failed"),
                    help_line("d v", "All filtered"),
                    help_line("d x", "All disabled"),
                ],
            ),
        ],
        [
            section(
                "Filters",
                vec![
                    help_line("T", "Toggle deleted"),
                    help_line("F", "Cycle filter"),
                    help_line("P", "Cycle protocol"),
                    help_line("S", "Cycle sort"),
                ],
            ),
            section(
                "Subscriptions",
                vec![
                    help_line("u", "Update all"),
                    help_line("r", "Refresh focused"),
                    help_line("n", "Rename"),
                    help_line("d", "Delete"),
                    help_line("y", "Show QR"),
                    help_line("c", "Copy link"),
                ],
            ),
            section(
                "Purge",
                vec![
                    help_line("D D", "Focused"),
                    help_line("D f", "All failed"),
                    help_line("D v", "Filtered trash"),
                    help_line("D a", "Empty trash"),
                ],
            ),
        ],
        [
            section(
                "API",
                vec![
                    help_line("a q", "Show API QR"),
                    help_line("a c", "Copy API link"),
                ],
            ),
            section(
                "Log",
                vec![
                    help_line("C l", "Clear log view"),
                    help_line("C s", "Clear traffic view"),
                    help_line("C p", "Clear events (db)"),
                ],
            ),
            section(
                "Restore",
                vec![
                    help_line("r r", "Focused"),
                    help_line("r v", "Filtered trash"),
                    help_line("r a", "All trash"),
                ],
            ),
        ],
    ];

    let row_count = grid.len();
    let row_heights: Vec<usize> = grid
        .iter()
        .map(|row| row.iter().map(Vec::len).max().unwrap_or(0))
        .collect();

    let mut cols: [Vec<Line>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for (row_index, row) in grid.into_iter().enumerate() {
        let target = row_heights[row_index];
        for (col_index, mut sect) in row.into_iter().enumerate() {
            while sect.len() < target {
                sect.push(Line::raw(""));
            }
            cols[col_index].append(&mut sect);
            if row_index + 1 < row_count {
                cols[col_index].push(Line::raw(""));
            }
        }
    }
    let [column_one, column_two, column_three] = cols;

    const GAP: u16 = 3;
    let column_width = |column: &[Line<'_>]| {
        column
            .iter()
            .map(|line| line.width() as u16)
            .max()
            .unwrap_or(0)
    };
    let widths = [
        column_width(&column_one),
        column_width(&column_two),
        column_width(&column_three),
    ];

    let docs = format!("  Docs: {}", env!("CARGO_PKG_HOMEPAGE"));
    let content_lines = column_one
        .len()
        .max(column_two.len())
        .max(column_three.len()) as u16;
    let inner_width = (widths.iter().sum::<u16>() + GAP * 2).max(docs.chars().count() as u16);
    // content + top/bottom borders + blank spacer + docs footer
    let height = (content_lines + 4).min(area.height);
    let area = centered_rect_fixed(inner_width + 2, height, area);

    frame.render_widget(Clear, area);
    let block = Block::default().title(" Help ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(content_lines),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(widths[0]),
            Constraint::Length(GAP),
            Constraint::Length(widths[1]),
            Constraint::Length(GAP),
            Constraint::Length(widths[2]),
        ])
        .split(rows[0]);

    frame.render_widget(
        Paragraph::new(column_one)
            .alignment(Alignment::Left)
            .style(theme::chrome_style()),
        columns[0],
    );
    frame.render_widget(
        Paragraph::new(column_two)
            .alignment(Alignment::Left)
            .style(theme::chrome_style()),
        columns[2],
    );
    frame.render_widget(
        Paragraph::new(column_three)
            .alignment(Alignment::Left)
            .style(theme::chrome_style()),
        columns[4],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  Docs: ", theme::muted_style()),
            Span::styled(env!("CARGO_PKG_HOMEPAGE"), theme::accent_style()),
        ])),
        rows[2],
    );
}

fn help_line<'a>(key: &'a str, description: &'a str) -> Line<'a> {
    // Pad by display width (not byte/char count) so keys containing wide or
    // multi-byte glyphs (arrows, combined `a / b`) still align their columns.
    const KEY_WIDTH: usize = 14;
    let pad = KEY_WIDTH.saturating_sub(UnicodeWidthStr::width(key));
    Line::from(vec![
        Span::raw("  "),
        Span::styled(
            format!("{key}{}", " ".repeat(pad)),
            theme::accent_style().bold(),
        ),
        Span::raw(description),
    ])
}

pub fn render_settings_modal(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let Some(modal) = &app.settings_modal else {
        return;
    };
    let width = area.width.saturating_sub(4).clamp(50, 110);
    let height = area.height.saturating_sub(2).clamp(20, 40);
    let modal_area = centered_rect_fixed(width, height, area);
    let compact = modal_area.width < 80;
    frame.render_widget(Clear, modal_area);

    let dirty = if modal.session.is_dirty() { " *" } else { "" };
    let title = format!(" Settings{dirty} · {} ", modal.session.path_display());
    let footer = settings_footer(modal.mode(), modal.pane, compact);
    let outer = Block::default()
        .title(Line::styled(title, theme::accent_style().bold()))
        .title_bottom(footer)
        .borders(Borders::ALL)
        .border_style(theme::muted_style())
        .padding(Padding::horizontal(1));
    let inner = outer.inner(modal_area);
    frame.render_widget(outer, modal_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(7),
            Constraint::Length(if compact { 11 } else { 10 }),
            Constraint::Length(
                if modal.error.is_some() || modal.notice.is_some() || modal.discard_confirm {
                    2
                } else if modal.selected_setting_index().is_none() {
                    1
                } else {
                    0
                },
            ),
        ])
        .split(inner);

    let (input_title, input_text) = if let Some(editing) = &modal.editing {
        let setting = &modal.session.settings[editing.setting_index];
        let text = if matches!(setting.kind, SettingKind::Secret) {
            "•".repeat(editing.input.chars().count())
        } else {
            editing.input.clone()
        };
        let unit = settings_value_unit(&setting.path)
            .map(|unit| format!(" · {unit}"))
            .unwrap_or_default();
        (
            format!(" Edit {}{unit} ", setting.label),
            format!("{text}█"),
        )
    } else if modal.searching {
        (" Search ".to_string(), format!("{}█", modal.query))
    } else {
        (
            " Search ".to_string(),
            if modal.query.is_empty() {
                "Press / to filter settings".to_string()
            } else {
                modal.query.clone()
            },
        )
    };
    frame.render_widget(
        Paragraph::new(input_text)
            .style(if modal.editing.is_some() || modal.searching {
                theme::accent_style()
            } else {
                theme::muted_style()
            })
            .block(
                Block::default()
                    .title(input_title)
                    .borders(Borders::ALL)
                    .border_style(if modal.editing.is_some() || modal.searching {
                        theme::accent_style()
                    } else {
                        theme::muted_style()
                    }),
            ),
        rows[0],
    );

    let columns = if compact {
        match modal.pane {
            SettingsPane::Sections => [rows[1], Rect::default()],
            SettingsPane::Fields => [Rect::default(), rows[1]],
        }
    } else {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(rows[1]);
        [columns[0], columns[1]]
    };
    let sections = modal.sections();
    let section_rows = columns[0].height.saturating_sub(2) as usize;
    let section_start = modal
        .section_index
        .saturating_sub(section_rows.saturating_sub(1));
    let section_lines: Vec<Line> = sections
        .iter()
        .enumerate()
        .skip(section_start)
        .take(section_rows)
        .map(|(index, section)| {
            let selected = index == modal.section_index;
            let marker = if selected { "› " } else { "  " };
            Line::styled(
                format!(
                    "{marker}{}",
                    settings_section_tree_label(section, &sections)
                ),
                if selected {
                    theme::accent_style().bold()
                } else {
                    theme::chrome_style()
                },
            )
        })
        .collect();
    if !columns[0].is_empty() {
        frame.render_widget(
            Paragraph::new(section_lines).block(
                Block::default()
                    .title(format!(
                        " Sections · {}/{} ",
                        modal.section_index.saturating_add(1).min(sections.len()),
                        sections.len()
                    ))
                    .borders(Borders::ALL)
                    .border_style(if modal.pane == SettingsPane::Sections {
                        theme::accent_style()
                    } else {
                        theme::muted_style()
                    }),
            ),
            columns[0],
        );
    }

    let indices = modal.visible_setting_indices();
    let available_rows = columns[1].height.saturating_sub(2) as usize;
    let section = modal.selected_section().unwrap_or_default();
    let mut last_group = String::new();
    let mut field_rows: Vec<(Option<usize>, Line)> = Vec::new();
    let mut selected_tail_rows = 0;
    let label_width = columns[1].width.saturating_sub(14).clamp(12, 24) as usize;
    for (position, setting_index) in indices.iter().enumerate() {
        let setting = &modal.session.settings[*setting_index];
        let group = settings_value_group(&section, &setting.section);
        if group != last_group {
            field_rows.push((
                None,
                Line::styled(format!("  {group}"), theme::muted_style().bold()),
            ));
            last_group = group;
        }
        let selected = position == modal.field_index;
        let state = settings_state_marker(setting);
        let marker = if selected { "›" } else { " " };
        let label_style = if selected {
            theme::accent_style().bold()
        } else {
            theme::chrome_style()
        };
        let value_style = if selected {
            theme::accent_style()
        } else {
            theme::muted_style()
        };
        if let Some((minimum, maximum)) = settings_range_pair(&setting.path, &setting.value) {
            field_rows.push((
                Some(position),
                Line::styled(format!("{marker}{state} {}", setting.label), label_style),
            ));
            for (label, value) in [("min", minimum), ("max", maximum)] {
                field_rows.push((
                    None,
                    Line::from(vec![
                        Span::styled(format!("    {label:<22}"), theme::chrome_style()),
                        Span::styled(settings_value_with_unit(&setting.path, value), value_style),
                    ]),
                ));
            }
            if selected {
                selected_tail_rows = 2;
            }
            continue;
        }
        let value = settings_value_display(
            &setting.path,
            &setting.value,
            matches!(setting.kind, SettingKind::Secret),
        );
        field_rows.push((
            Some(position),
            Line::from(vec![
                Span::styled(
                    format!("{marker}{state} {:<label_width$}", setting.label),
                    label_style,
                ),
                Span::styled(value, value_style),
            ]),
        ));
    }
    let selected_row = field_rows
        .iter()
        .position(|(position, _)| *position == Some(modal.field_index))
        .unwrap_or_default();
    let selected_row_end = selected_row + selected_tail_rows;
    let start = selected_row_end.saturating_sub(available_rows.saturating_sub(1));
    let field_lines: Vec<Line> = field_rows
        .into_iter()
        .skip(start)
        .take(available_rows)
        .map(|(_, line)| line)
        .collect();
    if !columns[1].is_empty() {
        frame.render_widget(
            Paragraph::new(field_lines).block(
                Block::default()
                    .title(format!(
                        " Values · {}/{} ",
                        modal.field_index.saturating_add(1).min(indices.len()),
                        indices.len()
                    ))
                    .borders(Borders::ALL)
                    .border_style(if modal.pane == SettingsPane::Fields {
                        theme::accent_style()
                    } else {
                        theme::muted_style()
                    }),
            ),
            columns[1],
        );
    }

    if let Some(index) = modal.selected_setting_index() {
        let setting = &modal.session.settings[index];
        let secret = matches!(setting.kind, SettingKind::Secret);
        let default_value = settings_value_display(&setting.path, setting.default_value(), secret);
        let (applies, applies_style) = (
            format!(
                "{} — {}",
                setting.effect.label(),
                setting.effect.help_text()
            ),
            theme::chrome_style(),
        );
        let help_lines = vec![
            Line::from(vec![
                Span::styled("Description  ", theme::accent_style().bold()),
                Span::styled(setting.help.description, theme::chrome_style()),
            ]),
            Line::from(vec![
                Span::styled("Values       ", theme::accent_style().bold()),
                Span::styled(setting.possible_values(), theme::chrome_style()),
            ]),
            Line::from(vec![
                Span::styled("Default      ", theme::accent_style().bold()),
                Span::styled(default_value, theme::chrome_style()),
            ]),
            Line::from(vec![
                Span::styled("Source       ", theme::accent_style().bold()),
                Span::styled(settings_source_display(setting), theme::chrome_style()),
            ]),
            Line::from(vec![
                Span::styled("Legend       ", theme::accent_style().bold()),
                Span::styled(
                    "· inherited default   + explicit override   * unsaved",
                    theme::muted_style(),
                ),
            ]),
            Line::from(vec![
                Span::styled("Example      ", theme::accent_style().bold()),
                Span::styled(setting.help.example, theme::chrome_style()),
            ]),
            Line::from(vec![
                Span::styled("Applies      ", theme::accent_style().bold()),
                Span::styled(applies, applies_style),
            ]),
        ];
        frame.render_widget(
            Paragraph::new(help_lines).wrap(Wrap { trim: true }).block(
                Block::default()
                    .title(format!(" Help · {} ", setting.path))
                    .borders(Borders::ALL)
                    .border_style(theme::muted_style()),
            ),
            rows[2],
        );
    } else {
        frame.render_widget(
            Paragraph::new("No settings match the filter.")
                .style(theme::muted_style())
                .block(
                    Block::default()
                        .title(" Help ")
                        .borders(Borders::ALL)
                        .border_style(theme::muted_style()),
                ),
            rows[2],
        );
    }

    let status = if modal.discard_confirm {
        Line::from(vec![
            Span::styled("Discard unsaved changes?  ", theme::failure_style().bold()),
            Span::styled("y", theme::success_style().bold()),
            Span::styled(" / ", theme::muted_style()),
            Span::styled("n", theme::failure_style().bold()),
        ])
    } else if let Some(error) = &modal.error {
        Line::styled(error.as_str(), theme::failure_style())
    } else if let Some(notice) = &modal.notice {
        Line::styled(notice.as_str(), theme::success_style())
    } else {
        Line::styled("No settings match the filter.", theme::muted_style())
    };
    frame.render_widget(Paragraph::new(status), rows[3]);
}

fn settings_footer(mode: SettingsMode, pane: SettingsPane, compact: bool) -> Line<'static> {
    let hints: &[(&str, &str)] = match mode {
        SettingsMode::DiscardConfirm => &[("y", "discard"), ("n/Esc", "keep")],
        SettingsMode::Search | SettingsMode::Edit if compact => &[
            ("Enter", "apply"),
            ("^U", "clear"),
            ("^S", "save"),
            ("Esc", "back"),
        ],
        SettingsMode::Search => &[
            ("Enter", "apply"),
            ("Ctrl+U", "clear"),
            ("Ctrl+S", "save"),
            ("Esc", "cancel"),
        ],
        SettingsMode::Edit => &[
            ("Enter", "apply"),
            ("Ctrl+U", "clear"),
            ("Ctrl+S", "save"),
            ("Esc", "cancel"),
        ],
        SettingsMode::Browse if compact => &[
            ("←/→", "pane"),
            ("Enter", "select"),
            ("^S", "save"),
            ("Esc", "close"),
        ],
        SettingsMode::Browse if pane == SettingsPane::Fields => &[
            ("←/→", "pane"),
            ("↑/↓", "move"),
            ("Enter", "edit"),
            ("r", "reset"),
            ("Ctrl+S", "save"),
            ("Esc", "close"),
        ],
        SettingsMode::Browse => &[
            ("←/→", "pane"),
            ("↑/↓", "move"),
            ("Enter", "open"),
            ("/", "search"),
            ("Ctrl+S", "save"),
            ("Esc", "close"),
        ],
    };
    let mut spans = Vec::with_capacity(hints.len() * 2);
    for (key, action) in hints {
        spans.push(Span::styled(
            format!(" {key}"),
            theme::accent_style().bold(),
        ));
        spans.push(Span::styled(format!(" {action} "), theme::muted_style()));
    }
    Line::from(spans).right_aligned()
}

fn settings_state_marker(setting: &EditableSetting) -> &'static str {
    if setting.is_dirty() {
        "*"
    } else if setting.is_explicit() {
        "+"
    } else {
        "·"
    }
}

fn settings_source_display(setting: &EditableSetting) -> String {
    let source = if setting.is_explicit() {
        "explicit override"
    } else {
        "inherited default"
    };
    if setting.is_reset() {
        format!("{source} · reset to default on save · unsaved")
    } else if setting.is_dirty() {
        format!("{source} · unsaved")
    } else {
        source.to_string()
    }
}

fn settings_section_tree_label(section: &str, sections: &[String]) -> String {
    let parts: Vec<&str> = section.split('.').collect();
    let depth = parts.len().saturating_sub(1);
    let mut label = String::new();

    for ancestor_depth in 1..depth {
        let ancestor = &parts[..=ancestor_depth];
        label.push_str(if is_last_section_child(ancestor, sections) {
            "   "
        } else {
            "│  "
        });
    }
    if depth > 0 {
        label.push_str(if is_last_section_child(&parts, sections) {
            "└─ "
        } else {
            "├─ "
        });
    }
    label.push_str(&settings_section_name(
        parts.last().copied().unwrap_or(section),
    ));
    label
}

fn is_last_section_child(parts: &[&str], sections: &[String]) -> bool {
    sections
        .iter()
        .rev()
        .find(|candidate| {
            let candidate_parts: Vec<&str> = candidate.split('.').collect();
            candidate_parts.len() == parts.len()
                && candidate_parts[..parts.len().saturating_sub(1)]
                    == parts[..parts.len().saturating_sub(1)]
        })
        .is_some_and(|candidate| candidate == &parts.join("."))
}

fn settings_section_name(segment: &str) -> String {
    segment
        .split('_')
        .map(|word| match word {
            "api" | "dns" | "http" | "https" | "icmp" | "tcp" => word.to_ascii_uppercase(),
            "auth" => "Authentication".to_string(),
            "geoip" => "GeoIP".to_string(),
            _ => {
                let mut chars = word.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                    .unwrap_or_default()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn settings_value_group(page: &str, section: &str) -> String {
    let suffix = section
        .strip_prefix(page)
        .unwrap_or(section)
        .trim_start_matches('.');
    if suffix.is_empty() {
        "General".to_string()
    } else {
        suffix
            .split('.')
            .map(settings_section_name)
            .collect::<Vec<_>>()
            .join(" › ")
    }
}

fn settings_value_display(path: &str, value: &SettingValue, secret: bool) -> String {
    let display = match value {
        SettingValue::Bool(true) => "✓".to_string(),
        SettingValue::Bool(false) => "✗".to_string(),
        SettingValue::List(values) if values.is_empty() => "none".to_string(),
        SettingValue::Integer(0)
            if matches!(
                path,
                "testing.concurrency" | "runtime.rotation.test_concurrency"
            ) =>
        {
            "auto".to_string()
        }
        value => value.display(secret),
    };
    if matches!(value, SettingValue::Integer(_)) {
        settings_value_with_unit(path, &display)
    } else {
        display
    }
}

fn settings_value_with_unit(path: &str, value: &str) -> String {
    let Some(unit) = settings_value_unit(path) else {
        return value.to_string();
    };
    format!("{value} {unit}")
}

fn settings_value_unit(path: &str) -> Option<&'static str> {
    match path {
        "runtime.fragment.interval"
        | "testing.download.timeout"
        | "testing.icmp.timeout"
        | "testing.real_delay.timeout"
        | "testing.tcp.timeout"
        | "testing.geoip.remote.timeout_ms" => Some("ms"),
        "runtime.rotation.cooldown_secs"
        | "runtime.rotation.interval_secs"
        | "testing.geoip.cache.ttl_secs" => Some("s"),
        "subscriptions.refresh_interval_hours" => Some("h"),
        _ => None,
    }
}

fn settings_range_pair<'a>(path: &str, value: &'a SettingValue) -> Option<(&'a str, &'a str)> {
    if !matches!(
        path,
        "runtime.fragment.packets" | "runtime.fragment.length" | "runtime.fragment.interval"
    ) {
        return None;
    }
    let SettingValue::List(values) = value else {
        return None;
    };
    match values.as_slice() {
        [minimum, maximum] => Some((minimum, maximum)),
        _ => None,
    }
}

pub fn render_import_modal(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let Some(modal) = &app.import_modal else {
        return;
    };

    const WIDTH: u16 = 72;
    const CONTENT_PADDING: u16 = 2;

    let (title, hint, placeholder, submit_label) = match &modal.step {
        ImportModalStep::Link => (
            " Import config or subscription ",
            "Paste one config link or HTTP(S) subscription URL.",
            "vless://… or https://…",
            "continue",
        ),
        ImportModalStep::SubscriptionName { suggested_name, .. } => (
            " Name subscription ",
            "Enter a name, or leave blank to use the suggestion.",
            suggested_name.as_str(),
            "import",
        ),
    };
    let has_error = modal.error.is_some();
    let modal_height = if has_error { 8 } else { 7 };
    let modal_area = centered_rect_fixed(WIDTH.min(area.width), modal_height, area);

    frame.render_widget(Clear, modal_area);
    let footer = Line::from(vec![
        Span::styled(" Enter", theme::accent_style().bold()),
        Span::styled(format!(" {submit_label}   "), theme::muted_style()),
        Span::styled("Esc", theme::accent_style().bold()),
        Span::styled(" cancel ", theme::muted_style()),
    ])
    .right_aligned();
    let outer = Block::default()
        .title(Line::styled(title, theme::accent_style().bold()))
        .title_bottom(footer)
        .borders(Borders::ALL)
        .border_style(theme::muted_style())
        .padding(Padding::horizontal(CONTENT_PADDING));
    let inner = outer.inner(modal_area);
    frame.render_widget(outer, modal_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if has_error {
            vec![
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
            ]
        } else {
            vec![Constraint::Length(1), Constraint::Length(3)]
        })
        .split(inner);
    frame.render_widget(Paragraph::new(hint).style(theme::muted_style()), rows[0]);

    let input = if modal.input.is_empty() {
        Line::from(vec![
            Span::styled(placeholder, theme::muted_style()),
            Span::styled("█", theme::accent_style()),
        ])
    } else {
        Line::styled(format!("{}█", modal.input), theme::accent_style())
    };
    let visible_input_width = rows[1].width.saturating_sub(4);
    let input_scroll = (UnicodeWidthStr::width(modal.input.as_str()) as u16 + 1)
        .saturating_sub(visible_input_width);
    frame.render_widget(
        Paragraph::new(input).scroll((0, input_scroll)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme::accent_style())
                .padding(Padding::horizontal(1)),
        ),
        rows[1],
    );

    if let Some(error) = &modal.error {
        frame.render_widget(
            Paragraph::new(error.as_str()).style(theme::failure_style()),
            rows[2],
        );
    }
}

pub fn render_rename_modal(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let Some(modal) = &app.rename_modal else {
        return;
    };

    const MIN_WIDTH: u16 = 42;
    const MAX_WIDTH: u16 = 72;
    const CONTENT_PADDING: u16 = 2;

    let title = format!(" Rename {} · {} ", modal.source_ref, modal.current_name);
    let input_width = UnicodeWidthStr::width(modal.input.as_str()) as u16 + 5;
    let content_width = (UnicodeWidthStr::width(title.as_str()) as u16)
        .max(input_width)
        .max(30);
    let modal_width = (content_width + CONTENT_PADDING * 2 + 2).clamp(MIN_WIDTH, MAX_WIDTH);
    let has_error = modal.error.is_some();
    let modal_height = if has_error { 6 } else { 5 };
    let modal_area = centered_rect_fixed(modal_width, modal_height, area);

    frame.render_widget(Clear, modal_area);
    let footer = Line::from(vec![
        Span::styled(" Enter", theme::accent_style().bold()),
        Span::styled(" save   ", theme::muted_style()),
        Span::styled("Esc", theme::accent_style().bold()),
        Span::styled(" cancel ", theme::muted_style()),
    ])
    .right_aligned();
    let outer = Block::default()
        .title(Line::styled(title, theme::accent_style().bold()))
        .title_bottom(footer)
        .borders(Borders::ALL)
        .border_style(theme::muted_style())
        .padding(Padding::horizontal(CONTENT_PADDING));
    let inner = outer.inner(modal_area);
    frame.render_widget(outer, modal_area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if has_error {
            vec![Constraint::Length(3), Constraint::Length(1)]
        } else {
            vec![Constraint::Length(3)]
        })
        .split(inner);

    let input = if modal.input.is_empty() {
        Line::from(vec![
            Span::styled("Subscription name", theme::muted_style()),
            Span::styled("█", theme::accent_style()),
        ])
    } else {
        Line::styled(format!("{}█", modal.input), theme::accent_style())
    };
    let visible_input_width = rows[0].width.saturating_sub(4);
    let input_scroll = (UnicodeWidthStr::width(modal.input.as_str()) as u16 + 1)
        .saturating_sub(visible_input_width);
    frame.render_widget(
        Paragraph::new(input).scroll((0, input_scroll)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme::accent_style())
                .padding(Padding::horizontal(1)),
        ),
        rows[0],
    );

    if let Some(error) = &modal.error {
        frame.render_widget(
            Paragraph::new(error.as_str()).style(theme::failure_style()),
            rows[1],
        );
    }
}

pub fn render_qr_modal(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let Some(modal) = &app.qr_modal else {
        return;
    };

    // Render the QR code first so its module count defines the modal width.
    let mut qr_lines: Vec<Line> = Vec::new();
    match qrcode::QrCode::with_error_correction_level(modal.uri.as_bytes(), qrcode::EcLevel::L) {
        Ok(code) => {
            let width = code.width();
            let pixels = code.to_colors();
            let mut row = 0usize;
            while row < width {
                let mut spans: Vec<Span> = Vec::new();
                for col in 0..width {
                    let top = pixels[row * width + col] == qrcode::Color::Dark;
                    let bot = if row + 1 < width {
                        pixels[(row + 1) * width + col] == qrcode::Color::Dark
                    } else {
                        false
                    };
                    let ch = match (top, bot) {
                        (true, true) => '█',
                        (true, false) => '▀',
                        (false, true) => '▄',
                        (false, false) => ' ',
                    };
                    spans.push(Span::raw(ch.to_string()));
                }
                qr_lines.push(Line::from(spans));
                row += 2;
            }
        }
        Err(_) => {
            qr_lines.push(Line::styled(
                "QR generation failed (URI may be too long)",
                theme::failure_style(),
            ));
        }
    }

    const PAD: u16 = 4;
    let vertical_pad = PAD / 2;
    let qr_width = qr_lines
        .iter()
        .map(|line| line.width() as u16)
        .max()
        .unwrap_or(0);
    let label_width = modal.label.chars().count() as u16;
    let content_width = qr_width.max(label_width);
    let inner_width = content_width + PAD * 2;
    // qr rows + label. With half-block rendering, width is roughly 2x height
    // in terminal cells, which keeps the modal visually square.
    let content_height = qr_lines.len() as u16 + 1 + vertical_pad * 2;

    let modal_area = centered_rect_fixed(inner_width + 2, content_height + 2, area);
    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(format!(" {} ", modal.kind.modal_title()))
        .borders(Borders::ALL);
    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let mut lines = Vec::new();
    for _ in 0..vertical_pad {
        lines.push(Line::raw(""));
    }
    lines.extend(qr_lines);
    lines.push(Line::styled(modal.label.clone(), theme::muted_style()));
    for _ in 0..vertical_pad {
        lines.push(Line::raw(""));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .style(theme::chrome_style()),
        inner,
    );
}

/// Centered rect with a fixed width and height (in cells), clamped to `area`.
pub fn centered_rect_fixed(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;
    use crate::app::config::ConfigEditSession;
    use crate::tui::app::{
        RenameModalState, SettingsEditState, SettingsModalState, SettingsPane, TuiAction,
    };

    #[test]
    fn rename_modal_identifies_subscription_by_ref_and_name() {
        let app = TuiApp {
            rename_modal: Some(RenameModalState {
                source_id: 7,
                source_ref: "sub-a1b2c3".to_string(),
                current_name: "Primary".to_string(),
                input: "Primary".to_string(),
                error: None,
            }),
            ..TuiApp::default()
        };
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

        terminal
            .draw(|frame| render_rename_modal(frame, frame.area(), &app))
            .unwrap();

        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("Rename sub-a1b2c3 · Primary"));
        assert!(rendered.contains("Enter save   Esc cancel"));
        assert!(!rendered.contains("New subscription name"));
        assert!(!rendered.contains("Subscription #7"));
    }

    #[test]
    fn settings_modal_never_renders_secret_plaintext() {
        let root = tempfile::tempdir().expect("temp directory should be created");
        let path = root.path().join("config.toml");
        fs::write(&path, "[server]\nkey = \"top-secret\"\n").expect("config should be written");
        let session = ConfigEditSession::open(&path).expect("settings should open");
        let secret_index = session
            .settings
            .iter()
            .position(|setting| setting.path == "server.key")
            .expect("secret setting should exist");
        let mut modal = SettingsModalState::new(session);
        modal.section_index = modal
            .sections()
            .iter()
            .position(|section| section == "server")
            .expect("server section should exist");
        modal.field_index = modal
            .visible_setting_indices()
            .iter()
            .position(|index| *index == secret_index)
            .expect("secret field should be visible");
        modal.editing = Some(SettingsEditState {
            setting_index: secret_index,
            input: "replacement".to_string(),
        });
        let app = TuiApp {
            settings_modal: Some(modal),
            ..TuiApp::default()
        };
        let mut terminal = Terminal::new(TestBackend::new(110, 34)).unwrap();

        terminal
            .draw(|frame| render_settings_modal(frame, frame.area(), &app))
            .unwrap();

        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(!rendered.contains("top-secret"));
        assert!(!rendered.contains("replacement"));
        assert!(rendered.contains("configured"));
        assert!(rendered.contains("XRAT_API_KEY"));
    }

    #[test]
    fn settings_sections_render_as_capitalized_tree_rows() {
        let sections = [
            "runtime",
            "runtime.http",
            "runtime.socks",
            "runtime.stats",
            "testing",
            "testing.download",
        ]
        .map(str::to_string);

        let labels: Vec<String> = sections
            .iter()
            .map(|section| settings_section_tree_label(section, &sections))
            .collect();

        assert_eq!(
            labels,
            [
                "Runtime",
                "├─ HTTP",
                "├─ Socks",
                "└─ Stats",
                "Testing",
                "└─ Download",
            ]
        );
        assert_eq!(
            settings_value_group("runtime.socks", "runtime.socks"),
            "General"
        );
        assert_eq!(
            settings_value_group("runtime.socks", "runtime.socks.auth"),
            "Authentication"
        );
        assert_eq!(settings_section_name("dns"), "DNS");
        assert_eq!(settings_section_tree_label("dns", &sections), "DNS");
        assert_eq!(
            settings_value_display(
                "runtime.sniffing.domains_excluded",
                &SettingValue::List(Vec::new()),
                false
            ),
            "none"
        );
        assert_eq!(
            settings_value_display("testing.concurrency", &SettingValue::Integer(0), false),
            "auto"
        );
        assert_eq!(
            settings_value_display(
                "runtime.rotation.test_concurrency",
                &SettingValue::Integer(0),
                false,
            ),
            "auto"
        );
        assert_eq!(
            settings_value_display(
                "runtime.mux.xudp_concurrency",
                &SettingValue::Integer(0),
                false,
            ),
            "0"
        );
        assert_eq!(
            settings_value_display(
                "testing.real_delay.timeout",
                &SettingValue::Integer(10_000),
                false,
            ),
            "10000 ms"
        );
        assert_eq!(
            settings_value_display(
                "runtime.rotation.interval_secs",
                &SettingValue::Integer(1800),
                false,
            ),
            "1800 s"
        );
        assert_eq!(
            settings_value_display(
                "subscriptions.refresh_interval_hours",
                &SettingValue::Integer(24),
                false,
            ),
            "24 h"
        );
        assert_eq!(
            settings_value_display("server.port", &SettingValue::Integer(18203), false),
            "18203"
        );
    }

    #[test]
    fn compact_settings_modal_shows_only_the_focused_pane() {
        let root = tempfile::tempdir().expect("temp directory should be created");
        let path = root.path().join("config.toml");
        fs::write(&path, "").expect("config should be written");
        let session = ConfigEditSession::open(&path).expect("settings should open");
        let mut app = TuiApp {
            settings_modal: Some(SettingsModalState::new(session)),
            ..TuiApp::default()
        };
        let mut terminal = Terminal::new(TestBackend::new(60, 30)).unwrap();

        terminal
            .draw(|frame| render_settings_modal(frame, frame.area(), &app))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("Sections ·"));
        assert!(!rendered.contains("Values ·"));
        assert!(rendered.contains("^S save"));
        assert!(rendered.contains("Esc close"));

        app.settings_modal.as_mut().expect("modal").pane = SettingsPane::Fields;
        terminal
            .draw(|frame| render_settings_modal(frame, frame.area(), &app))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(!rendered.contains("Sections ·"));
        assert!(rendered.contains("Values ·"));
    }

    #[test]
    fn settings_section_selection_remains_visible_in_short_modal() {
        let root = tempfile::tempdir().expect("temp directory should be created");
        let path = root.path().join("config.toml");
        fs::write(&path, "").expect("config should be written");
        let session = ConfigEditSession::open(&path).expect("settings should open");
        let mut modal = SettingsModalState::new(session);
        let sections = modal.sections();
        modal.section_index = sections.len().saturating_sub(1);
        let selected = modal.selected_section().expect("selected section");
        let selected_label = settings_section_tree_label(&selected, &sections);
        let app = TuiApp {
            settings_modal: Some(modal),
            ..TuiApp::default()
        };
        let mut terminal = Terminal::new(TestBackend::new(110, 22)).unwrap();

        terminal
            .draw(|frame| render_settings_modal(frame, frame.area(), &app))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains(&format!("› {selected_label}")));
    }

    #[test]
    fn settings_help_shows_origin_default_and_dns_effect() {
        let root = tempfile::tempdir().expect("temp directory should be created");
        let path = root.path().join("config.toml");
        fs::write(&path, "[dns]\nquery_strategy = \"UseSystem\"\n")
            .expect("config should be written");
        let session = ConfigEditSession::open(&path).expect("settings should open");
        let mut modal = SettingsModalState::new(session);
        modal.section_index = modal
            .sections()
            .iter()
            .position(|section| section == "dns")
            .expect("DNS section");
        modal.field_index = modal
            .visible_setting_indices()
            .iter()
            .position(|index| modal.session.settings[*index].path == "dns.query_strategy")
            .expect("query strategy setting");
        modal.pane = SettingsPane::Fields;
        let app = TuiApp {
            settings_modal: Some(modal),
            ..TuiApp::default()
        };
        let mut terminal = Terminal::new(TestBackend::new(110, 34)).unwrap();

        terminal
            .draw(|frame| render_settings_modal(frame, frame.area(), &app))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("DNS"));
        assert!(rendered.contains("Default"));
        assert!(rendered.contains("Source"));
        assert!(rendered.contains("Legend"));
        assert!(rendered.contains("+ explicit override"));
        assert!(rendered.contains("explicit override"));
        assert!(rendered.contains("runtime restart"));
    }

    #[test]
    fn settings_parent_page_renders_nested_value_subheaders() {
        let root = tempfile::tempdir().expect("temp directory should be created");
        let path = root.path().join("config.toml");
        fs::write(
            &path,
            "[runtime.socks]\nenabled = true\n[runtime.socks.auth]\nenabled = false\n",
        )
        .expect("config should be written");
        let session = ConfigEditSession::open(&path).expect("settings should open");
        let mut modal = SettingsModalState::new(session);
        modal.section_index = modal
            .sections()
            .iter()
            .position(|section| section == "runtime.socks")
            .expect("runtime.socks section should exist");
        let app = TuiApp {
            settings_modal: Some(modal),
            ..TuiApp::default()
        };
        let mut terminal = Terminal::new(TestBackend::new(110, 34)).unwrap();

        terminal
            .draw(|frame| render_settings_modal(frame, frame.area(), &app))
            .unwrap();

        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("General"));
        assert!(rendered.contains("Authentication"));
        assert!(rendered.contains('✓'));
        assert!(rendered.contains('✗'));
        assert!(!rendered.contains("runtime.socks.auth"));
    }

    #[test]
    fn settings_help_follows_selection_in_compact_terminal() {
        let root = tempfile::tempdir().expect("temp directory should be created");
        let path = root.path().join("config.toml");
        fs::write(&path, "[runtime.socks]\nenabled = true\nport = 18200\n")
            .expect("config should be written");
        let session = ConfigEditSession::open(&path).expect("settings should open");
        let mut modal = SettingsModalState::new(session);
        modal.section_index = modal
            .sections()
            .iter()
            .position(|section| section == "runtime.socks")
            .expect("runtime.socks section should exist");
        let port_index = modal
            .visible_setting_indices()
            .iter()
            .position(|index| modal.session.settings[*index].path == "runtime.socks.port")
            .expect("port setting should be visible");
        let enabled_index = modal
            .visible_setting_indices()
            .iter()
            .position(|index| modal.session.settings[*index].path == "runtime.socks.enabled")
            .expect("enabled setting should be visible");
        modal.field_index = port_index;
        modal.pane = SettingsPane::Fields;
        let mut app = TuiApp {
            settings_modal: Some(modal),
            ..TuiApp::default()
        };
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

        terminal
            .draw(|frame| render_settings_modal(frame, frame.area(), &app))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("Help · runtime.socks.port"));
        assert!(rendered.contains("Description"));
        assert!(rendered.contains("Values"));
        assert!(rendered.contains("Example"));
        assert!(rendered.contains("port = 18200"));
        assert!(rendered.contains("runtime restart"));

        let direction = if enabled_index < port_index { -1 } else { 1 };
        for _ in 0..enabled_index.abs_diff(port_index) {
            app.apply(TuiAction::SettingsMove(direction));
        }
        terminal
            .draw(|frame| render_settings_modal(frame, frame.area(), &app))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("Help · runtime.socks.enabled"));
        assert!(rendered.contains("✓ enabled · ✗ disabled"));
        assert!(rendered.contains("enabled = true"));
    }

    #[test]
    fn settings_fragment_ranges_render_as_indented_min_max_rows() {
        let root = tempfile::tempdir().expect("temp directory should be created");
        let path = root.path().join("config.toml");
        fs::write(
            &path,
            "[runtime.fragment]\npackets = [1, 3]\nlength = [100, 200]\ninterval = [10, 20]\n",
        )
        .expect("config should be written");
        let session = ConfigEditSession::open(&path).expect("settings should open");
        let mut modal = SettingsModalState::new(session);
        modal.section_index = modal
            .sections()
            .iter()
            .position(|section| section == "runtime.fragment")
            .expect("runtime.fragment section should exist");
        let app = TuiApp {
            settings_modal: Some(modal),
            ..TuiApp::default()
        };
        let mut terminal = Terminal::new(TestBackend::new(110, 34)).unwrap();

        terminal
            .draw(|frame| render_settings_modal(frame, frame.area(), &app))
            .unwrap();

        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.matches("min").count() >= 3);
        assert!(rendered.matches("max").count() >= 3);
        assert!(!rendered.contains("10, 20"));
        assert_eq!(
            settings_range_pair(
                "runtime.fragment.interval",
                &SettingValue::List(vec!["10".to_string(), "20".to_string()]),
            ),
            Some(("10", "20"))
        );
        assert_eq!(
            settings_value_with_unit("runtime.fragment.interval", "10"),
            "10 ms"
        );
    }
}
