use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// A terminal key event with optional layout-independent physical key information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiKeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    /// Base-layout key reported by modern terminal protocols (such as Kitty keyboard protocol).
    /// Represents the code point this physical key would produce on a standard US QWERTY base layout.
    pub base_layout_key: Option<char>,
    pub text: Option<String>,
}

impl TuiKeyEvent {
    #[allow(dead_code)]
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self {
            code,
            modifiers,
            base_layout_key: None,
            text: None,
        }
    }

    #[allow(dead_code)]
    pub const fn with_base_layout(mut self, base: char) -> Self {
        self.base_layout_key = Some(base);
        self
    }

    /// Normalizes this key event into a standard `KeyEvent` based on whether the current context
    /// is editing text (e.g. search query, rename field) or navigating.
    ///
    /// When `base_layout_key` is available:
    /// - In navigation/command mode: maps to the physical key position so shortcuts work
    ///   independently of the active layout (Cyrillic, Persian, Arabic, CJK, AZERTY, etc.).
    /// - In text input mode: preserves the typed character for input, but normalizes `Control`
    ///   modifier shortcuts (`Ctrl+C`, `Ctrl+O`, `Ctrl+W`, `Ctrl+U`, `Ctrl+S`).
    pub fn normalize(&self, is_text_input: bool) -> KeyEvent {
        let code = if let (KeyCode::Char(_), Some(base)) = (self.code, self.base_layout_key) {
            if !is_text_input || self.modifiers.contains(KeyModifiers::CONTROL) {
                KeyCode::Char(
                    if self.modifiers.contains(KeyModifiers::SHIFT)
                        && !self.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        shifted_us_key(base)
                    } else {
                        base
                    },
                )
            } else {
                self.code
            }
        } else {
            self.code
        };

        KeyEvent::new(code, self.modifiers)
    }
}

impl From<KeyEvent> for TuiKeyEvent {
    fn from(key: KeyEvent) -> Self {
        Self {
            code: key.code,
            modifiers: key.modifiers,
            base_layout_key: None,
            text: None,
        }
    }
}

impl From<&qwertty::event::KeyEvent> for TuiKeyEvent {
    fn from(ev: &qwertty::event::KeyEvent) -> Self {
        let mut modifiers = KeyModifiers::empty();
        let m = ev.modifiers();
        if m.contains(qwertty::Modifiers::SHIFT) {
            modifiers |= KeyModifiers::SHIFT;
        }
        if m.contains(qwertty::Modifiers::CTRL) {
            modifiers |= KeyModifiers::CONTROL;
        }
        if m.contains(qwertty::Modifiers::ALT) {
            modifiers |= KeyModifiers::ALT;
        }
        if m.contains(qwertty::Modifiers::SUPER) {
            modifiers |= KeyModifiers::SUPER;
        }

        let code = match ev.key() {
            qwertty::Key::Char(character)
                if ev.text().is_none() && ('\u{e061}'..='\u{e06e}').contains(&character) =>
            {
                KeyCode::Null
            }
            qwertty::Key::Char(character) => {
                let resolved = ev
                    .text()
                    .and_then(|text| {
                        let mut characters = text.as_str().chars();
                        let first = characters.next()?;
                        characters.next().is_none().then_some(first)
                    })
                    .unwrap_or_else(|| {
                        if modifiers.contains(KeyModifiers::SHIFT) {
                            ev.shifted_key()
                                .unwrap_or_else(|| shifted_us_key(character))
                        } else {
                            character
                        }
                    });
                KeyCode::Char(resolved)
            }
            qwertty::Key::Up => KeyCode::Up,
            qwertty::Key::Down => KeyCode::Down,
            qwertty::Key::Left => KeyCode::Left,
            qwertty::Key::Right => KeyCode::Right,
            qwertty::Key::Enter => KeyCode::Enter,
            qwertty::Key::Tab => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    KeyCode::BackTab
                } else {
                    KeyCode::Tab
                }
            }
            qwertty::Key::Backspace => KeyCode::Backspace,
            qwertty::Key::Escape => KeyCode::Esc,
            qwertty::Key::Home => KeyCode::Home,
            qwertty::Key::End => KeyCode::End,
            qwertty::Key::PageUp => KeyCode::PageUp,
            qwertty::Key::PageDown => KeyCode::PageDown,
            qwertty::Key::Delete => KeyCode::Delete,
            qwertty::Key::Function(n) => KeyCode::F(n),
            qwertty::Key::Control(b) => {
                if (1..=26).contains(&b) {
                    modifiers |= KeyModifiers::CONTROL;
                    KeyCode::Char((b'a' + (b - 1)) as char)
                } else {
                    KeyCode::Null
                }
            }
            _ => KeyCode::Null,
        };

        Self {
            code,
            modifiers,
            base_layout_key: ev.base_layout_key(),
            text: ev.text().map(|text| text.as_str().to_string()),
        }
    }
}

fn shifted_us_key(character: char) -> char {
    match character {
        'a'..='z' => character.to_ascii_uppercase(),
        '`' => '~',
        '1' => '!',
        '2' => '@',
        '3' => '#',
        '4' => '$',
        '5' => '%',
        '6' => '^',
        '7' => '&',
        '8' => '*',
        '9' => '(',
        '0' => ')',
        '-' => '_',
        '=' => '+',
        '[' => '{',
        ']' => '}',
        '\\' => '|',
        ';' => ':',
        '\'' => '"',
        ',' => '<',
        '.' => '>',
        '/' => '?',
        _ => character,
    }
}
