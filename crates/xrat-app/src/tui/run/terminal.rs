use std::io;
use std::time::Duration;

use crossterm::event::{
    DisableBracketedPaste, EnableBracketedPaste, KeyboardEnhancementFlags,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiEvent {
    Key(TuiKeyEvent),
    Resize(u16, u16),
    Paste(String),
}

pub struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    #[cfg(unix)]
    input: input::TerminalInput,
    last_size: (u16, u16),
}

impl TerminalSession {
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        let keyboard_flags = KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
            | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS;
        #[cfg(unix)]
        let keyboard_flags = keyboard_flags | REPORT_ASSOCIATED_TEXT;
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableBracketedPaste,
            PushKeyboardEnhancementFlags(keyboard_flags)
        )?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let initial_size = crossterm::terminal::size().unwrap_or((0, 0));

        Ok(Self {
            terminal,
            #[cfg(unix)]
            input: input::TerminalInput::default(),
            last_size: initial_size,
        })
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
            unix::poll_event(&mut self.input, timeout)
        }

        #[cfg(not(unix))]
        {
            if crossterm::event::poll(timeout)? {
                match crossterm::event::read()? {
                    crossterm::event::Event::Key(k) => {
                        return Ok(Some(TuiEvent::Key(TuiKeyEvent::from(k))));
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
            LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
        let _ = self.terminal.show_cursor();
    }
}
