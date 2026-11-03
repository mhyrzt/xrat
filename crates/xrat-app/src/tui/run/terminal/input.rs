use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::{TuiEvent, TuiMouseButton, TuiMouseEvent, TuiMouseEventKind};
use crate::tui::keymap::TuiKeyEvent;

const ESCAPE_TIMEOUT: Duration = Duration::from_millis(25);

fn convert_modifiers(m: qwertty::Modifiers) -> crossterm::event::KeyModifiers {
    let mut modifiers = crossterm::event::KeyModifiers::empty();
    if m.contains(qwertty::Modifiers::SHIFT) {
        modifiers |= crossterm::event::KeyModifiers::SHIFT;
    }
    if m.contains(qwertty::Modifiers::CTRL) {
        modifiers |= crossterm::event::KeyModifiers::CONTROL;
    }
    if m.contains(qwertty::Modifiers::ALT) {
        modifiers |= crossterm::event::KeyModifiers::ALT;
    }
    if m.contains(qwertty::Modifiers::SUPER) {
        modifiers |= crossterm::event::KeyModifiers::SUPER;
    }
    modifiers
}

#[derive(Default)]
pub(super) struct TerminalInput {
    decoder: qwertty::SemanticDecoder,
    events: VecDeque<TuiEvent>,
    escape_since: Option<Instant>,
}

impl TerminalInput {
    pub(super) fn feed(&mut self, bytes: &[u8]) {
        let events = self.decoder.feed(bytes);
        self.extend(events);
    }

    pub(super) fn settle(&mut self, now: Instant) {
        if self.decoder.has_settled_text() {
            let events = self.decoder.finish();
            self.extend(events);
        }
        if self.decoder.has_pending_lone_escape() {
            self.escape_since.get_or_insert(now);
        } else {
            self.escape_since = None;
        }
    }

    pub(super) fn wait_duration(&self, timeout: Duration, now: Instant) -> Duration {
        self.escape_since.map_or(timeout, |since| {
            timeout.min(ESCAPE_TIMEOUT.saturating_sub(now.duration_since(since)))
        })
    }

    pub(super) fn pop(&mut self, now: Instant) -> Option<TuiEvent> {
        if self
            .escape_since
            .is_some_and(|since| now.duration_since(since) >= ESCAPE_TIMEOUT)
        {
            if let Some(event) = self.decoder.flush_pending_escape() {
                self.extend([event]);
            }
            self.escape_since = None;
        }
        self.events.pop_front()
    }

    fn extend(&mut self, events: impl IntoIterator<Item = qwertty::Event>) {
        self.events
            .extend(events.into_iter().filter_map(|event| match event {
                qwertty::Event::Key(key) if key.kind() != qwertty::event::KeyEventKind::Release => {
                    Some(TuiEvent::Key(TuiKeyEvent::from(&key)))
                }
                qwertty::Event::Resize(resize) => {
                    let cells = resize.cells();
                    Some(TuiEvent::Resize(cells.columns(), cells.rows()))
                }
                qwertty::Event::Paste(paste) => {
                    paste.as_str().map(|text| TuiEvent::Paste(text.to_string()))
                }
                qwertty::Event::Mouse(mouse) => {
                    let kind = match mouse.kind() {
                        qwertty::event::MouseEventKind::Press => match mouse.button() {
                            qwertty::event::MouseButton::Left => {
                                TuiMouseEventKind::Down(TuiMouseButton::Left)
                            }
                            qwertty::event::MouseButton::Middle => {
                                TuiMouseEventKind::Down(TuiMouseButton::Middle)
                            }
                            qwertty::event::MouseButton::Right => {
                                TuiMouseEventKind::Down(TuiMouseButton::Right)
                            }
                            _ => return None,
                        },
                        qwertty::event::MouseEventKind::Release => match mouse.button() {
                            qwertty::event::MouseButton::Left => {
                                TuiMouseEventKind::Up(TuiMouseButton::Left)
                            }
                            qwertty::event::MouseButton::Middle => {
                                TuiMouseEventKind::Up(TuiMouseButton::Middle)
                            }
                            qwertty::event::MouseButton::Right => {
                                TuiMouseEventKind::Up(TuiMouseButton::Right)
                            }
                            _ => return None,
                        },
                        qwertty::event::MouseEventKind::Moved => return None,
                        qwertty::event::MouseEventKind::Scroll(dir) => match dir {
                            qwertty::event::ScrollDirection::Up => TuiMouseEventKind::ScrollUp,
                            qwertty::event::ScrollDirection::Down => TuiMouseEventKind::ScrollDown,
                            qwertty::event::ScrollDirection::Left => TuiMouseEventKind::ScrollLeft,
                            qwertty::event::ScrollDirection::Right => {
                                TuiMouseEventKind::ScrollRight
                            }
                            _ => return None,
                        },
                        _ => return None,
                    };
                    let column = mouse.column().saturating_sub(1);
                    let row = mouse.row().saturating_sub(1);
                    let modifiers = convert_modifiers(mouse.modifiers());
                    Some(TuiEvent::Mouse(TuiMouseEvent {
                        kind,
                        column,
                        row,
                        modifiers,
                    }))
                }
                _ => None,
            }));
    }
}

#[cfg(test)]
mod tests;
