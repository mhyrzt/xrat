use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::services::release_notes::ReleaseNotes;
use crate::tui::theme;

pub(crate) struct ReleaseNotesModal {
    pub notes: ReleaseNotes,
    text: Text<'static>,
    palette: theme::ThemePalette,
    scroll: usize,
    max_scroll: usize,
    page: usize,
}

impl ReleaseNotesModal {
    pub fn new(notes: ReleaseNotes) -> Self {
        Self {
            text: markdown(&notes.body),
            palette: theme::palette(),
            notes,
            scroll: 0,
            max_scroll: 0,
            page: 1,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind == KeyEventKind::Release {
            return false;
        }
        match key.code {
            KeyCode::Enter | KeyCode::Esc => return true,
            KeyCode::Down | KeyCode::Char('j') => self.scroll = self.scroll.saturating_add(1),
            KeyCode::Up | KeyCode::Char('k') => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::PageDown | KeyCode::Char(' ') => {
                self.scroll = self.scroll.saturating_add(self.page)
            }
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(self.page),
            KeyCode::Home | KeyCode::Char('g') => self.scroll = 0,
            KeyCode::End | KeyCode::Char('G') => self.scroll = self.max_scroll,
            _ => {}
        }
        self.scroll = self.scroll.min(self.max_scroll);
        false
    }

    pub fn handle_scroll(&mut self, delta: isize) {
        if delta < 0 {
            self.scroll = self.scroll.saturating_sub(delta.unsigned_abs());
        } else {
            self.scroll = (self.scroll + delta as usize).min(self.max_scroll);
        }
    }

    pub fn render(&mut self, frame: &mut Frame<'_>) {
        let palette = theme::palette();
        if self.palette != palette {
            self.text = markdown(&self.notes.body);
            self.palette = palette;
        }
        let screen = frame.area();
        let width = screen.width.saturating_sub(4).min(100);
        let height = screen.height.saturating_sub(2);
        let area = Rect::new(
            screen.x + (screen.width - width) / 2,
            screen.y + (screen.height - height) / 2,
            width,
            height,
        );
        crate::tui::theme::clear(frame, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::accent_style())
            .style(theme::surface_style())
            .title(format!(" What's new · {} ", self.notes.version))
            .title_bottom(" ↑↓ / j k · PgUp/PgDn · Home/End · Enter/Esc close ");
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let body = Rect::new(
            inner.x.saturating_add(1),
            inner.y,
            inner.width.saturating_sub(2),
            inner.height.saturating_sub(1),
        );
        let paragraph = Paragraph::new(self.text.clone()).wrap(Wrap { trim: false });
        let count = paragraph.line_count(body.width);
        self.page = usize::from(body.height).max(1);
        self.max_scroll = count
            .saturating_sub(usize::from(body.height))
            .min(u16::MAX as usize);
        self.scroll = self.scroll.min(self.max_scroll);
        frame.render_widget(paragraph.scroll((self.scroll as u16, 0)), body);
        if inner.height > 0 {
            let footer = format!(
                "Lines {}–{} of {count}",
                self.scroll + 1,
                (self.scroll + self.page).min(count)
            );
            frame.render_widget(
                Paragraph::new(footer).style(theme::muted_style()),
                Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1),
            );
        }
    }
}

fn markdown(source: &str) -> Text<'static> {
    let clean: String = source
        .chars()
        .filter(|ch| !ch.is_control() || matches!(ch, '\n' | '\t'))
        .collect();
    let mut lines = Vec::new();
    let mut spans = Vec::new();
    let mut styles = vec![Style::default()];
    let flush = |lines: &mut Vec<Line<'static>>, spans: &mut Vec<Span<'static>>| {
        lines.push(Line::from(std::mem::take(spans)));
    };
    for event in Parser::new_ext(
        &clean,
        Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES,
    ) {
        match event {
            Event::Start(tag) => {
                let base = *styles.last().unwrap();
                let style = match &tag {
                    Tag::Heading { .. } => base
                        .patch(theme::accent_style())
                        .add_modifier(Modifier::BOLD),
                    Tag::Strong => base.add_modifier(Modifier::BOLD),
                    Tag::Emphasis => base.add_modifier(Modifier::ITALIC),
                    Tag::Strikethrough => base.add_modifier(Modifier::CROSSED_OUT),
                    Tag::CodeBlock(_) => base.patch(theme::warning_style()),
                    Tag::Link { .. } => base
                        .patch(theme::secondary_style())
                        .add_modifier(Modifier::UNDERLINED),
                    _ => base,
                };
                if matches!(tag, Tag::Item) {
                    spans.push(Span::raw("  • "));
                }
                if matches!(tag, Tag::BlockQuote(_)) {
                    spans.push(Span::styled("│ ", theme::muted_style()));
                }
                styles.push(style);
            }
            Event::End(tag) => {
                styles.pop();
                if matches!(
                    tag,
                    TagEnd::Paragraph
                        | TagEnd::Heading(_)
                        | TagEnd::CodeBlock
                        | TagEnd::Item
                        | TagEnd::TableRow
                ) {
                    flush(&mut lines, &mut spans);
                    if matches!(
                        tag,
                        TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::CodeBlock
                    ) {
                        lines.push(Line::default());
                    }
                }
                if tag == TagEnd::TableCell {
                    spans.push(Span::raw("  │  "));
                }
            }
            Event::Text(text) | Event::Html(text) | Event::InlineHtml(text) => {
                for (index, part) in text.split('\n').enumerate() {
                    if index > 0 {
                        flush(&mut lines, &mut spans);
                    }
                    spans.push(Span::styled(
                        part.to_string(),
                        *styles.last().unwrap_or(&Style::default()),
                    ));
                }
            }
            Event::Code(text) => {
                spans.push(Span::styled(text.into_string(), theme::warning_style()))
            }
            Event::SoftBreak => spans.push(Span::raw(" ")),
            Event::HardBreak => flush(&mut lines, &mut spans),
            Event::Rule => {
                flush(&mut lines, &mut spans);
                lines.push(Line::styled(
                    "────────────────────────",
                    theme::muted_style(),
                ));
            }
            Event::TaskListMarker(done) => {
                spans.push(Span::raw(if done { "☑ " } else { "☐ " }))
            }
            _ => {}
        }
    }
    if !spans.is_empty() {
        flush(&mut lines, &mut spans);
    }
    Text::from(lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn long_markdown_scrolls_to_end_and_resize_clamps_offset() {
        let notes = ReleaseNotes {
            version: "v1.2.3".into(),
            body: "# Changes\n\n".to_string() + &"- **Fixed** `something`\n".repeat(150),
            seen: false,
        };
        let mut modal = ReleaseNotesModal::new(notes);
        let mut terminal = Terminal::new(TestBackend::new(60, 20)).unwrap();
        terminal.draw(|frame| modal.render(frame)).unwrap();
        assert!(modal.max_scroll > 100);
        assert!(!modal.handle_key(KeyEvent::new(
            KeyCode::End,
            crossterm::event::KeyModifiers::NONE
        )));
        assert_eq!(modal.scroll, modal.max_scroll);
        terminal.draw(|frame| modal.render(frame)).unwrap();
        assert!(modal.handle_key(KeyEvent::new(
            KeyCode::Esc,
            crossterm::event::KeyModifiers::NONE
        )));
        assert!(!modal.notes.seen);
        terminal.backend_mut().resize(4, 2);
        terminal.autoresize().unwrap();
        terminal.draw(|frame| modal.render(frame)).unwrap();
    }

    #[test]
    fn malformed_markdown_remains_readable_and_control_codes_are_removed() {
        let _guard = theme::TEST_THEME_LOCK.lock().unwrap();
        let text = markdown("# Heading\n\n**unfinished [link\n\n```rust\ncode\u{1b}[31m\n");
        let plain = text.to_string();
        assert!(plain.contains("Heading"));
        assert!(plain.contains("unfinished"));
        assert!(plain.contains("code"));
        assert!(!plain.contains('\u{1b}'));
        assert!(
            text.lines[0]
                .spans
                .iter()
                .any(|span| span.style.fg == theme::accent_style().fg)
        );
    }
}
