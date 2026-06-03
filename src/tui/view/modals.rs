use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::theme;

pub fn render_help(frame: &mut Frame<'_>, area: Rect) {
    frame.render_widget(Clear, area);
    let block = Block::default().title(" Help ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let left = vec![
        Line::styled("Navigation", theme::muted_style()),
        help_line("1/2/3/4", "configs / sources / tests / diagnostics"),
        help_line("Tab", "cycle modes"),
        help_line("j/k, ↑/↓", "move focus"),
        help_line("Esc", "close modal or go back"),
        help_line("q, Ctrl+C", "quit"),
        Line::raw(""),
        Line::styled("Configs", theme::muted_style()),
        help_line("Enter", "select and start focused config"),
        help_line("Space", "select focused config"),
        help_line("e/x", "enable / disable focused"),
        help_line("E/X", "enable / disable selected"),
        help_line("d/D/r", "soft delete / purge / restore"),
        help_line("y/c/C", "QR / copy focused / copy selected"),
    ];

    let right = vec![
        Line::styled("Search and Filters", theme::muted_style()),
        help_line("/", "edit config search"),
        help_line("Ctrl+U", "clear search while editing"),
        help_line("f/F/P/s", "deleted / filter / protocol / sort"),
        Line::raw(""),
        Line::styled("Tests", theme::muted_style()),
        help_line("t", "test current scope from Configs"),
        help_line("a/v", "test all enabled / visible filtered"),
        help_line("s/c", "start / cancel batch in Tests"),
        Line::raw(""),
        Line::styled("Runtime", theme::muted_style()),
        help_line("K/R", "kill / restart"),
        Line::raw(""),
        Line::styled("Sources", theme::muted_style()),
        help_line("r/R", "refresh focused / refresh all"),
        help_line("i/n/d", "import / rename / delete"),
        help_line("y/c/u/U", "QR / copy / QR API / copy API"),
    ];

    frame.render_widget(
        Paragraph::new(left)
            .alignment(Alignment::Left)
            .style(theme::chrome_style()),
        columns[0],
    );
    frame.render_widget(
        Paragraph::new(right)
            .alignment(Alignment::Left)
            .style(theme::chrome_style()),
        columns[1],
    );
}

fn help_line<'a>(key: &'a str, description: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{key:<12}"), theme::accent_style().bold()),
        Span::raw(description),
    ])
}

pub fn render_confirm(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let Some(confirm) = &app.confirm else {
        return;
    };

    frame.render_widget(Clear, area);
    let text = vec![
        Line::styled(&confirm.message, theme::chrome_style()),
        Line::raw(""),
        Line::styled("Enter/y confirm   Esc/n cancel", theme::muted_style()),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .title(confirm.title.as_str())
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}

pub fn render_import_modal(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let Some(modal) = &app.import_modal else {
        return;
    };

    frame.render_widget(Clear, area);
    let cursor_input = format!("{}█", modal.input);
    let mut lines = vec![
        Line::styled(
            "Paste or type a subscription URL, file path, or raw config text.",
            theme::muted_style(),
        ),
        Line::raw(""),
        Line::styled(&cursor_input, theme::accent_style()),
    ];
    if let Some(err) = &modal.error {
        lines.push(Line::raw(""));
        lines.push(Line::styled(err.as_str(), theme::failure_style()));
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "Enter import   Esc cancel",
        theme::muted_style(),
    ));

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Import / Add Source ")
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .style(theme::chrome_style())
            .wrap(Wrap { trim: false }),
        area,
    );
}

pub fn render_rename_modal(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let Some(modal) = &app.rename_modal else {
        return;
    };

    frame.render_widget(Clear, area);
    let cursor_input = format!("{}█", modal.input);
    let mut lines = vec![
        Line::styled(
            "Enter a new name for this subscription.",
            theme::muted_style(),
        ),
        Line::raw(""),
        Line::styled(&cursor_input, theme::accent_style()),
    ];
    if let Some(err) = &modal.error {
        lines.push(Line::raw(""));
        lines.push(Line::styled(err.as_str(), theme::failure_style()));
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "Enter save   Esc cancel",
        theme::muted_style(),
    ));

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(format!(" Rename Source #{} ", modal.source_id))
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .style(theme::chrome_style())
            .wrap(Wrap { trim: false }),
        area,
    );
}

pub fn render_qr_modal(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let Some(modal) = &app.qr_modal else {
        return;
    };

    frame.render_widget(Clear, area);

    let mut lines: Vec<Line> = Vec::new();
    match qrcode::QrCode::new(modal.uri.as_bytes()) {
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
                lines.push(Line::from(spans));
                row += 2;
            }
        }
        Err(_) => {
            lines.push(Line::styled(
                "QR generation failed (URI may be too long)",
                theme::failure_style(),
            ));
        }
    }
    lines.push(Line::raw(""));
    let uri_preview = if modal.uri.len() > 60 {
        format!("{}…", &modal.uri[..60])
    } else {
        modal.uri.clone()
    };
    lines.push(Line::styled(uri_preview, theme::muted_style()));
    lines.push(Line::raw(""));
    lines.push(Line::styled("Esc/q close", theme::muted_style()));

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(format!(" QR: {} ", modal.title))
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .style(theme::chrome_style()),
        area,
    );
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}
