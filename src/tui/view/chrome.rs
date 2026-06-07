use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::tui::app::TuiApp;
use crate::tui::keymap;
use crate::tui::theme;

pub fn render_key_bar(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
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

    let actions = [
        Span::styled("Keybindings: ", theme::chrome_style()),
        Span::styled("?", theme::accent_style().bold()),
    ];

    let center = center_spans(app);
    let center_width: usize = center.iter().map(Span::width).sum();

    let total = area.width as usize;
    let brand_width: usize = brand.iter().map(Span::width).sum();
    let actions_width: usize = actions.iter().map(Span::width).sum();
    let remaining = total
        .saturating_sub(brand_width + actions_width + center_width + theme::EDGE_MARGIN)
        .max(3);
    let left_gap = remaining / 2;
    let right_gap = remaining - left_gap;

    let mut spans = brand;
    spans.push(Span::raw(" ".repeat(left_gap)));
    spans.extend(center);
    spans.push(Span::raw(" ".repeat(right_gap)));
    spans.extend(actions);

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Center segment of the key bar. Priority: an inline destructive confirm
/// (single-row or bulk — there are no confirm modals), then an armed chord
/// hint.
fn center_spans(app: &TuiApp) -> Vec<Span<'static>> {
    if let Some(confirm) = &app.confirm {
        return confirm_prompt(confirm.prompt.clone());
    }
    if let Some(op) = app.pending_bulk {
        let count = app.bulk_config_ids(op).len();
        return confirm_prompt(format!("{} {count} {} configs?", op.verb(), op.target()));
    }
    if let Some(leader) = app.pending_chord {
        return chord_hint(leader);
    }
    Vec::new()
}

/// `<prompt>  y / n` with the prompt in danger red and the keys colour-coded.
fn confirm_prompt(text: String) -> Vec<Span<'static>> {
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
            theme::accent_style().bold(),
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
