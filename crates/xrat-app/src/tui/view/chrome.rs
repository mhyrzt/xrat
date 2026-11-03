use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::tui::app::TuiApp;
use crate::tui::keymap;
use crate::tui::theme;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KeyBarAction {
    Settings,
    Help,
}

pub fn render_key_bar(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let (spans, _) = key_bar_content(area.width, app);
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub(crate) fn key_bar_action(area: Rect, app: &TuiApp, column: u16) -> Option<KeyBarAction> {
    let (_, actions_start) = key_bar_content(area.width, app);
    let offset = usize::from(column.checked_sub(area.x)?);
    let start = actions_start?;
    if offset >= usize::from(area.width) {
        return None;
    }
    if (start..start + "Settings: ,".len()).contains(&offset) {
        Some(KeyBarAction::Settings)
    } else if (start + "Settings: ,  ".len()..start + "Settings: ,  Keybindings: ?".len())
        .contains(&offset)
    {
        Some(KeyBarAction::Help)
    } else {
        None
    }
}

fn key_bar_content(width: u16, app: &TuiApp) -> (Vec<Span<'static>>, Option<usize>) {
    let mut brand = vec![Span::styled(
        concat!("XRAT v", env!("CARGO_PKG_VERSION")),
        theme::accent_style().bold(),
    )];
    if let Some(latest) = &app.latest_version {
        brand.push(Span::styled(" → ", theme::muted_style()));
        brand.push(Span::styled(
            format!("v{}", latest.trim_start_matches('v')),
            theme::success_style().bold(),
        ));
    }

    brand.push(Span::raw("  "));
    if app.data.runtime.pid_running
        && (app.data.runtime.status == "running"
            || app.data.runtime.status == "connected"
            || app.data.runtime.status == "degraded")
    {
        let active = app
            .data
            .runtime
            .active_config
            .as_deref()
            .or(app.data.runtime.session_config.as_deref())
            .unwrap_or("connected");
        let active_display = if active.chars().count() > 24 {
            let truncated: String = active.chars().take(23).collect();
            format!("{truncated}…")
        } else {
            active.to_string()
        };
        let dot_style = if app.data.runtime.status == "degraded" {
            theme::warning_style().bold()
        } else {
            theme::success_style().bold()
        };
        let mode = app.data.runtime.mode_label();
        brand.push(Span::styled("● ", dot_style));
        brand.push(Span::styled(active_display, theme::success_style()));
        brand.push(Span::styled(format!(" [{mode}]"), theme::muted_style()));
    } else if app.runtime_activity_in_flight() {
        brand.push(Span::styled(
            format!("{} connecting", app.spinner_frame()),
            theme::warning_style().bold(),
        ));
    } else {
        brand.push(Span::styled("○ idle", theme::muted_style()));
    }

    let actions = [
        Span::styled("Settings: ", theme::muted_style()),
        Span::styled(",", theme::accent_style().bold()),
        Span::raw("  "),
        Span::styled("Keybindings: ", theme::muted_style()),
        Span::styled("?", theme::accent_style().bold()),
    ];

    let total = width as usize;
    let center = center_spans(app, total);
    let center_width: usize = center.iter().map(Span::width).sum();

    let brand_width: usize = brand.iter().map(Span::width).sum();
    let actions_width: usize = actions.iter().map(Span::width).sum();

    let center_fits = brand_width + actions_width + center_width + theme::EDGE_MARGIN <= total;
    let essential = app.confirm.is_some()
        || app.pending_bulk.is_some()
        || app.pending_chord.is_some()
        || app.chrome_message.is_some();
    if !center_fits && essential {
        return (center, None);
    }

    let (center, center_width) = if center_fits {
        (center, center_width)
    } else {
        (Vec::new(), 0)
    };

    let remaining = total
        .saturating_sub(brand_width + actions_width + center_width + theme::EDGE_MARGIN)
        .max(1);
    let left_gap = remaining / 2;
    let right_gap = remaining - left_gap;

    let mut spans = brand;
    spans.push(Span::raw(" ".repeat(left_gap)));
    spans.extend(center);
    spans.push(Span::raw(" ".repeat(right_gap)));
    let actions_start = spans.iter().map(Span::width).sum();
    spans.extend(actions);
    (spans, Some(actions_start))
}

/// Center segment of the key bar. Priority: an inline destructive confirm
/// (single-row or bulk — there are no confirm modals), then an armed chord
/// hint, then transient messages, then contextual shortcuts.
fn center_spans(app: &TuiApp, width: usize) -> Vec<Span<'static>> {
    if let Some(confirm) = &app.confirm {
        return confirm_prompt(confirm.prompt.clone(), width);
    }
    if let Some(op) = app.pending_bulk {
        let count = app.bulk_config_ids(op).len();
        return confirm_prompt(
            format!("{} {count} {} configs?", op.verb(), op.target()),
            width,
        );
    }
    if let Some(leader) = app.pending_chord {
        return chord_hint(leader);
    }
    if let Some(message) = &app.chrome_message {
        let style = if message.is_error {
            theme::failure_style().bold()
        } else {
            theme::success_style().bold()
        };
        return vec![Span::styled(message.text.clone(), style)];
    }
    contextual_hints(app)
}

fn contextual_hints(app: &TuiApp) -> Vec<Span<'static>> {
    let hints: &[(&str, &str)] = match app.active_view {
        crate::tui::app::TuiView::Configs => match app.focused_panel {
            crate::tui::app::TuiPanel::Table => &[
                ("↵", "connect"),
                ("t", "test"),
                ("/", "search"),
                ("s", "sort"),
                ("y", "QR"),
                ("c", "copy"),
                ("d", "del"),
                ("⇥", "panes"),
            ],
            crate::tui::app::TuiPanel::Detail => &[
                ("j/k", "scroll"),
                ("y", "QR"),
                ("c", "copy"),
                ("e/x", "toggle"),
                ("⇥", "panes"),
            ],
            crate::tui::app::TuiPanel::Log => &[
                ("[/]", "tabs"),
                ("C", "clear"),
                ("j/k", "scroll"),
                ("⇥", "panes"),
            ],
            crate::tui::app::TuiPanel::Runtime => &[
                ("K", "stop"),
                ("R", "restart"),
                ("U", "TUN"),
                ("G", "split"),
                ("⇥", "panes"),
            ],
        },
        crate::tui::app::TuiView::Sources => &[
            ("↵", "view"),
            ("r", "refresh"),
            ("u", "update all"),
            ("n", "rename"),
            ("d", "del"),
            ("y", "QR"),
            ("c", "copy"),
            ("⇥", "panes"),
        ],
    };

    let mut spans = Vec::with_capacity(hints.len() * 3);
    for (index, (key, label)) in hints.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" · ", theme::muted_style()));
        }
        spans.push(Span::styled(*key, theme::accent_style().bold()));
        spans.push(Span::styled(format!(" {label}"), theme::muted_style()));
    }
    spans
}

/// `<prompt>  y / n` with the prompt in danger red and the keys colour-coded.
fn confirm_prompt(text: String, width: usize) -> Vec<Span<'static>> {
    let prompt_width = width.saturating_sub("  y / n".width());
    let text = if text.width() > prompt_width {
        let mut shortened = String::new();
        let mut used = 0;
        for character in text.chars() {
            let character_width = character.width().unwrap_or(0);
            if used + character_width > prompt_width.saturating_sub(1) {
                break;
            }
            shortened.push(character);
            used += character_width;
        }
        if prompt_width > 0 {
            shortened.push('…');
        }
        shortened
    } else {
        text
    };
    vec![
        Span::styled(text, theme::failure_style().bold()),
        Span::raw("  "),
        Span::styled("y", theme::success_style().bold()),
        Span::styled(" / ", theme::muted_style()),
        Span::styled("n", theme::failure_style().bold()),
    ]
}

/// `LEADER  key label · key label · …  esc cancel`, keys highlighted.
fn chord_hint(leader: char) -> Vec<Span<'static>> {
    let mut spans = vec![
        Span::styled(
            keymap::chord_title(leader).to_string(),
            theme::secondary_style().bold(),
        ),
        Span::raw("  "),
    ];
    for (index, (key, label)) in keymap::chord_entries(leader).iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" · ", theme::muted_style()));
        }
        spans.push(Span::styled(
            (*key).to_string(),
            theme::accent_style().bold(),
        ));
        spans.push(Span::styled(format!(" {label}"), theme::chrome_style()));
    }
    spans.push(Span::styled("   esc cancel", theme::muted_style()));
    spans
}

#[cfg(test)]
#[path = "chrome_tests.rs"]
mod tests;
