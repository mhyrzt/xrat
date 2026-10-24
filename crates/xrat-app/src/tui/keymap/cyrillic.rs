use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Maps Cyrillic (Russian, Ukrainian, Belarusian) layout characters to their US QWERTY equivalents.
pub fn map_cyrillic_char(c: char) -> char {
    match c {
        // Lowercase
        'й' => 'q',
        'ц' => 'w',
        'у' => 'e',
        'к' => 'r',
        'е' => 't',
        'н' => 'y',
        'г' => 'u',
        'ш' => 'i',
        'щ' => 'o',
        'з' => 'p',
        'х' => '[',
        'ъ' => ']',
        'ф' => 'a',
        'ы' => 's',
        'в' => 'd',
        'а' => 'f',
        'п' => 'g',
        'р' => 'h',
        'о' => 'j',
        'л' => 'k',
        'д' => 'l',
        'ж' => ';',
        'э' => '\'',
        'я' => 'z',
        'ч' => 'x',
        'с' => 'c',
        'м' => 'v',
        'и' => 'b',
        'т' => 'n',
        'ь' => 'm',
        'б' => ',',
        'ю' => '.',
        'ё' => '`',

        // Uppercase (e.g. Shift + key)
        'Й' => 'Q',
        'Ц' => 'W',
        'У' => 'E',
        'К' => 'R',
        'Е' => 'T',
        'Н' => 'Y',
        'Г' => 'U',
        'Ш' => 'I',
        'Щ' => 'O',
        'З' => 'P',
        'Х' => '{',
        'Ъ' => '}',
        'Ф' => 'A',
        'Ы' => 'S',
        'В' => 'D',
        'А' => 'F',
        'П' => 'G',
        'Р' => 'H',
        'О' => 'J',
        'Л' => 'K',
        'Д' => 'L',
        'Ж' => ':',
        'Э' => '"',
        'Я' => 'Z',
        'Ч' => 'X',
        'С' => 'C',
        'М' => 'V',
        'И' => 'B',
        'Т' => 'N',
        'Ь' => 'M',
        'Б' => '<',
        'Ю' => '>',
        'Ё' => '~',

        // Ukrainian / Belarusian specific letters
        'і' => 's',
        'І' => 'S',
        'ї' => ']',
        'Ї' => '}',
        'є' => '\'',
        'Є' => '"',
        'ґ' => 'g',
        'Ґ' => 'G',
        'ў' => 'u',
        'Ў' => 'U',

        other => other,
    }
}

/// Normalizes a `KeyEvent` so that Cyrillic characters are mapped to their Latin QWERTY
/// equivalents.
///
/// When `is_text_input` is true (e.g. typing search query, rename, or input modal),
/// only shortcuts with the `Control` modifier are normalized, allowing users to type
/// Cyrillic text normally while still using shortcuts like `Ctrl+C`, `Ctrl+U`, `Ctrl+W`.
pub fn normalize_key(key: KeyEvent, is_text_input: bool) -> KeyEvent {
    if let KeyCode::Char(ch) = key.code {
        if is_text_input {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                return KeyEvent {
                    code: KeyCode::Char(map_cyrillic_char(ch)),
                    ..key
                };
            }
            return key;
        }

        return KeyEvent {
            code: KeyCode::Char(map_cyrillic_char(ch)),
            ..key
        };
    }
    key
}
