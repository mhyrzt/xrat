use std::io;
use std::time::Duration;

#[cfg(not(unix))]
use crossterm::event::EnableMouseCapture;
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::tui::keymap::TuiKeyEvent;

#[cfg(unix)]
mod input;
#[cfg(unix)]
mod unix;

// Crossterm leaves Kitty's associated-text flag unnamed because its events cannot carry text.
#[cfg(unix)]
const REPORT_ASSOCIATED_TEXT: KeyboardEnhancementFlags =
    KeyboardEnhancementFlags::from_bits_retain(0b0001_0000);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TuiMouseEvent {
    pub kind: TuiMouseEventKind,
    pub column: u16,
    pub row: u16,
    pub modifiers: crossterm::event::KeyModifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiMouseEventKind {
    Down(TuiMouseButton),
    Up(TuiMouseButton),
    #[allow(dead_code)]
    Drag(TuiMouseButton),
    #[allow(dead_code)]
    Moved,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiMouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiEvent {
    Key(TuiKeyEvent),
    Mouse(TuiMouseEvent),
    Resize(u16, u16),
    Paste(String),
}

pub struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    #[cfg(unix)]
    input: input::TerminalInput,
    last_size: (u16, u16),
    mouse_enabled: bool,
}

#[cfg(unix)]
struct EnableNormalMouseCapture;

#[cfg(unix)]
impl crossterm::Command for EnableNormalMouseCapture {
    fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        // Mode 1000: Report button press and release only (no cell motion / drag).
        // Mode 1006: SGR extended coordinates format.
        // We intentionally omit mode 1002 (cell motion) to prevent floods of drag events.
        write!(f, "\x1B[?1000h\x1B[?1006h")
    }
}

impl TerminalSession {
    pub fn enter(mouse_enabled: bool) -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        let keyboard_flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS;
        #[cfg(unix)]
        let keyboard_flags = keyboard_flags | REPORT_ASSOCIATED_TEXT;
        if mouse_enabled {
            #[cfg(unix)]
            execute!(
                stdout,
                EnterAlternateScreen,
                EnableBracketedPaste,
                PushKeyboardEnhancementFlags(keyboard_flags),
                EnableNormalMouseCapture
            )?;
            #[cfg(not(unix))]
            execute!(
                stdout,
                EnterAlternateScreen,
                EnableBracketedPaste,
                PushKeyboardEnhancementFlags(keyboard_flags),
                EnableMouseCapture
            )?;
        } else {
            execute!(
                stdout,
                EnterAlternateScreen,
                EnableBracketedPaste,
                PushKeyboardEnhancementFlags(keyboard_flags)
            )?;
        }
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let initial_size = crossterm::terminal::size().unwrap_or((0, 0));

        Ok(Self {
            terminal,
            #[cfg(unix)]
            input: input::TerminalInput::default(),
            last_size: initial_size,
            mouse_enabled,
        })
    }

    pub fn size(&self) -> (u16, u16) {
        self.last_size
    }

    pub fn mouse_enabled(&self) -> bool {
        self.mouse_enabled
    }

    pub fn set_mouse_capture(&mut self, enabled: bool) -> io::Result<()> {
        if self.mouse_enabled == enabled {
            return Ok(());
        }
        if enabled {
            #[cfg(unix)]
            execute!(self.terminal.backend_mut(), EnableNormalMouseCapture)?;
            #[cfg(not(unix))]
            execute!(self.terminal.backend_mut(), EnableMouseCapture)?;
        } else {
            execute!(self.terminal.backend_mut(), DisableMouseCapture)?;
        }
        self.mouse_enabled = enabled;
        Ok(())
    }

    pub fn draw<F>(&mut self, render: F) -> io::Result<()>
    where
        F: FnOnce(&mut ratatui::Frame<'_>),
    {
        self.terminal.draw(render).map(|_| ())
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.terminal.clear()
    }

    pub fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<TuiEvent>> {
        // First check for terminal geometry changes (window resize).
        if let Ok(current_size) = crossterm::terminal::size()
            && current_size != self.last_size
        {
            self.last_size = current_size;
            return Ok(Some(TuiEvent::Resize(current_size.0, current_size.1)));
        }

        #[cfg(unix)]
        {
            let mut current_timeout = timeout;
            loop {
                let event = unix::poll_event(&mut self.input, current_timeout)?;
                if matches!(event, Some(TuiEvent::Mouse(_))) && !self.mouse_enabled {
                    current_timeout = Duration::ZERO;
                    continue;
                }
                return Ok(event);
            }
        }

        #[cfg(not(unix))]
        {
            let mut current_timeout = timeout;
            while crossterm::event::poll(current_timeout)? {
                match crossterm::event::read()? {
                    crossterm::event::Event::Key(k) => {
                        return Ok(Some(TuiEvent::Key(TuiKeyEvent::from(k))));
                    }
                    crossterm::event::Event::Mouse(_) if !self.mouse_enabled => {
                        current_timeout = Duration::ZERO;
                        continue;
                    }
                    crossterm::event::Event::Mouse(m) => {
                        let kind = match m.kind {
                            crossterm::event::MouseEventKind::Down(btn) => match btn {
                                crossterm::event::MouseButton::Left => {
                                    TuiMouseEventKind::Down(TuiMouseButton::Left)
                                }
                                crossterm::event::MouseButton::Middle => {
                                    TuiMouseEventKind::Down(TuiMouseButton::Middle)
                                }
                                crossterm::event::MouseButton::Right => {
                                    TuiMouseEventKind::Down(TuiMouseButton::Right)
                                }
                            },
                            crossterm::event::MouseEventKind::Up(btn) => match btn {
                                crossterm::event::MouseButton::Left => {
                                    TuiMouseEventKind::Up(TuiMouseButton::Left)
                                }
                                crossterm::event::MouseButton::Middle => {
                                    TuiMouseEventKind::Up(TuiMouseButton::Middle)
                                }
                                crossterm::event::MouseButton::Right => {
                                    TuiMouseEventKind::Up(TuiMouseButton::Right)
                                }
                            },
                            crossterm::event::MouseEventKind::Drag(_)
                            | crossterm::event::MouseEventKind::Moved => return Ok(None),
                            crossterm::event::MouseEventKind::ScrollDown => {
                                TuiMouseEventKind::ScrollDown
                            }
                            crossterm::event::MouseEventKind::ScrollUp => {
                                TuiMouseEventKind::ScrollUp
                            }
                            crossterm::event::MouseEventKind::ScrollLeft => {
                                TuiMouseEventKind::ScrollLeft
                            }
                            crossterm::event::MouseEventKind::ScrollRight => {
                                TuiMouseEventKind::ScrollRight
                            }
                        };
                        return Ok(Some(TuiEvent::Mouse(TuiMouseEvent {
                            kind,
                            column: m.column,
                            row: m.row,
                            modifiers: m.modifiers,
                        })));
                    }
                    crossterm::event::Event::Resize(w, h) => {
                        return Ok(Some(TuiEvent::Resize(w, h)));
                    }
                    crossterm::event::Event::Paste(text) => {
                        return Ok(Some(TuiEvent::Paste(text)));
                    }
                    _ => {}
                }
            }
        }

        #[cfg(not(unix))]
        Ok(None)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = execute!(
            self.terminal.backend_mut(),
            PopKeyboardEnhancementFlags,
            DisableBracketedPaste,
            DisableMouseCapture,
            LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
        let _ = self.terminal.show_cursor();
    }
}
