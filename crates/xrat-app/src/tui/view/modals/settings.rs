use super::prelude::*;
use crate::tui::theme;

pub fn render_settings_modal(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let Some(modal) = &app.settings_modal else {
        return;
    };
    let width = area.width.saturating_sub(4).clamp(50, 110);
    let height = area.height.saturating_sub(2).clamp(20, 40);
    let modal_area = centered_rect_fixed(width, height, area);
    let compact = modal_area.width < 80;
    crate::tui::theme::clear(frame, modal_area);

    let dirty = if modal.session.is_dirty() { " *" } else { "" };
    let title = format!(" Settings{dirty} · {} ", modal.session.path_display());
    let footer = settings_footer(modal.mode(), modal.pane, compact);
    let outer = Block::default()
        .title(Line::styled(title, theme::accent_style().bold()))
        .title_bottom(footer)
        .borders(Borders::ALL)
        .border_type(theme::border_type())
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
    let label_width = columns[1].width.saturating_sub(14).clamp(12, 24) as usize;
    for (position, setting_index) in indices.iter().enumerate() {
        let setting = &modal.session.settings[*setting_index];
        let group = settings_value_group(&section, &setting.section);
        if group != last_group {
            field_rows.push((
                None,
                Line::styled(format!("  {group}"), theme::secondary_style().bold()),
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
                    Some(position),
                    Line::from(vec![
                        Span::styled(format!("    {label:<22}"), theme::chrome_style()),
                        Span::styled(settings_value_with_unit(&setting.path, value), value_style),
                    ]),
                ));
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
    let layout = settings_field_layout(modal, available_rows);
    let start = layout.start;
    let field_lines: Vec<Line> = if indices.is_empty() {
        vec![
            Line::raw(""),
            Line::styled(
                " No settings match search. Press Esc to reset.",
                theme::muted_style(),
            ),
        ]
    } else {
        field_rows
            .into_iter()
            .skip(start)
            .take(available_rows)
            .map(|(_, line)| line)
            .collect()
    };
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
                Span::styled("Description  ", theme::secondary_style().bold()),
                Span::styled(setting.help.description, theme::chrome_style()),
            ]),
            Line::from(vec![
                Span::styled("Values       ", theme::secondary_style().bold()),
                Span::styled(setting.possible_values(), theme::chrome_style()),
            ]),
            Line::from(vec![
                Span::styled("Default      ", theme::secondary_style().bold()),
                Span::styled(default_value, theme::chrome_style()),
            ]),
            Line::from(vec![
                Span::styled("Source       ", theme::secondary_style().bold()),
                Span::styled(settings_source_display(setting), theme::chrome_style()),
            ]),
            Line::from(vec![
                Span::styled("Legend       ", theme::secondary_style().bold()),
                Span::styled(
                    "· inherited default   + explicit override   * unsaved",
                    theme::muted_style(),
                ),
            ]),
            Line::from(vec![
                Span::styled("Example      ", theme::secondary_style().bold()),
                Span::styled(setting.help.example, theme::chrome_style()),
            ]),
            Line::from(vec![
                Span::styled("Applies      ", theme::secondary_style().bold()),
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

pub(crate) fn settings_footer(
    mode: SettingsMode,
    pane: SettingsPane,
    compact: bool,
) -> Line<'static> {
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

pub(crate) fn settings_state_marker(setting: &EditableSetting) -> &'static str {
    if setting.is_dirty() {
        "*"
    } else if setting.is_explicit() {
        "+"
    } else {
        "·"
    }
}

pub(crate) fn settings_source_display(setting: &EditableSetting) -> String {
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

pub(crate) fn settings_section_tree_label(section: &str, sections: &[String]) -> String {
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

pub(crate) fn is_last_section_child(parts: &[&str], sections: &[String]) -> bool {
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

pub(crate) fn settings_section_name(segment: &str) -> String {
    segment
        .split('_')
        .map(|word| match word {
            "api" | "dns" | "http" | "https" | "icmp" | "tcp" | "tui" | "ui" => {
                word.to_ascii_uppercase()
            }
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

pub(crate) fn settings_value_group(page: &str, section: &str) -> String {
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

pub(crate) fn settings_value_display(path: &str, value: &SettingValue, secret: bool) -> String {
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

pub(crate) fn settings_value_with_unit(path: &str, value: &str) -> String {
    let Some(unit) = settings_value_unit(path) else {
        return value.to_string();
    };
    format!("{value} {unit}")
}

pub(crate) fn settings_value_unit(path: &str) -> Option<&'static str> {
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

pub(crate) fn settings_range_pair<'a>(
    path: &str,
    value: &'a SettingValue,
) -> Option<(&'a str, &'a str)> {
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

#[derive(Debug, Clone)]
pub(crate) struct SettingsFieldLayout {
    pub rows: Vec<Option<usize>>,
    pub start: usize,
}

pub(crate) fn settings_field_layout(
    modal: &SettingsModalState,
    available_rows: usize,
) -> SettingsFieldLayout {
    let indices = modal.visible_setting_indices();
    let section = modal.selected_section().unwrap_or_default();
    let mut last_group = String::new();
    let mut rows: Vec<Option<usize>> = Vec::new();
    let mut selected_tail_rows = 0;
    for (position, setting_index) in indices.iter().enumerate() {
        let setting = &modal.session.settings[*setting_index];
        let group = settings_value_group(&section, &setting.section);
        if group != last_group {
            rows.push(None);
            last_group = group;
        }
        let selected = position == modal.field_index;
        if let Some((_minimum, _maximum)) = settings_range_pair(&setting.path, &setting.value) {
            rows.push(Some(position));
            rows.push(Some(position));
            rows.push(Some(position));
            if selected {
                selected_tail_rows = 2;
            }
            continue;
        }
        rows.push(Some(position));
    }
    let selected_row = rows
        .iter()
        .position(|position| *position == Some(modal.field_index))
        .unwrap_or_default();
    let selected_row_end = selected_row + selected_tail_rows;
    let start = selected_row_end.saturating_sub(available_rows.saturating_sub(1));
    SettingsFieldLayout { rows, start }
}
