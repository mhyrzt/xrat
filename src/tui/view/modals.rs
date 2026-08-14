use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph};
use unicode_width::UnicodeWidthStr;

use crate::tui::app::{ImportModalStep, TuiApp};
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
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;
    use crate::tui::app::RenameModalState;

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
}
