use super::prelude::*;
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
                vec![
                    help_line("K", "Kill"),
                    help_line("R", "Restart"),
                    help_line("U", "Toggle TUN capture"),
                    help_line("G", "Split tunneling"),
                ],
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
