use super::prelude::*;
use crate::tui::theme;

pub fn render_import_modal(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let Some(modal) = &app.import_modal else {
        return;
    };

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
            "Enter a name, or leave blank to detect it from the subscription.",
            suggested_name.as_str(),
            "import",
        ),
    };
    let has_error = modal.error.is_some();
    let modal_area = import_modal_area(area, has_error);

    crate::tui::theme::clear(frame, modal_area);
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
        .border_type(theme::border_type())
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

    const CONTENT_PADDING: u16 = 2;
    let title = format!(" Rename {} · {} ", modal.source_ref, modal.current_name);
    let modal_area = rename_modal_area(area, modal);
    let has_error = modal.error.is_some();

    crate::tui::theme::clear(frame, modal_area);
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
        .border_type(theme::border_type())
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
    let modal_area = qr_modal_area(area, modal);
    crate::tui::theme::clear(frame, modal_area);

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

pub(crate) fn import_modal_area(area: Rect, has_error: bool) -> Rect {
    centered_rect_fixed(72, if has_error { 8 } else { 7 }, area)
}

pub(crate) fn rename_modal_area(area: Rect, modal: &crate::tui::app::RenameModalState) -> Rect {
    let title = format!(" Rename {} · {} ", modal.source_ref, modal.current_name);
    let input_width = UnicodeWidthStr::width(modal.input.as_str()) as u16 + 5;
    let content_width = (UnicodeWidthStr::width(title.as_str()) as u16)
        .max(input_width)
        .max(30);
    let width = (content_width + 6).clamp(42, 72);
    centered_rect_fixed(width, if modal.error.is_some() { 6 } else { 5 }, area)
}

pub(crate) fn qr_modal_area(area: Rect, modal: &crate::tui::app::QrModalState) -> Rect {
    let (qr_width, qr_height) =
        match qrcode::QrCode::with_error_correction_level(modal.uri.as_bytes(), qrcode::EcLevel::L)
        {
            Ok(code) => (code.width() as u16, (code.width() as u16).div_ceil(2)),
            Err(_) => ("QR generation failed (URI may be too long)".len() as u16, 1),
        };
    let content_width = qr_width.max(modal.label.chars().count() as u16);
    centered_rect_fixed(content_width + 10, qr_height + 7, area)
}
