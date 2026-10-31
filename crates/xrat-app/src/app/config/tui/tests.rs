use super::*;
use ratatui::style::Color;

#[test]
fn inherits_and_overrides_custom_colors() {
    let settings: TuiSettings = toml::from_str(
        r##"
        theme = "custom"
        [themes.custom]
        extends = "catppuccin"
        background = "#010203"
        foreground = "terminal"
        selection_foreground = "white"
    "##,
    )
    .unwrap();
    let palette = settings.resolve(&settings.theme).unwrap();
    assert_eq!(palette.background, Color::Rgb(1, 2, 3));
    assert_eq!(palette.foreground, Color::Reset);
    assert_eq!(palette.selection_foreground, Color::White);
    assert_eq!(palette.accent, TuiTheme::Catppuccin.palette().accent);
    let serialized = toml::to_string(&settings).unwrap();
    assert_eq!(
        toml::from_str::<TuiSettings>(&serialized).unwrap(),
        settings
    );
}

#[test]
fn omitted_base_inherits_default_and_aliases_normalize() {
    let settings: TuiSettings = toml::from_str("theme = 'custom'\n[themes.custom]\n").unwrap();
    assert_eq!(
        settings.resolve(&settings.theme).unwrap(),
        TuiTheme::Default.palette()
    );
    let settings: TuiSettings = toml::from_str("theme = ' Mocha '").unwrap();
    assert_eq!(settings.theme, TuiTheme::Catppuccin.into());
}

#[test]
fn rejects_invalid_custom_definitions_and_selections() {
    for input in [
        "theme = 'missing'",
        "[themes.nord]",
        "[themes.mocha]",
        "[themes.'invalid.name']",
        "[themes.custom]\nextends = 'another-custom'",
        "[themes.custom]\naccent = '#12345g'",
        "[themes.custom]\nforeground = '#fff'",
        "[themes.custom]\nforeground = 42",
        "[themes.custom]\nunknown = 'red'",
        "themes = 'wrong-type'",
    ] {
        assert!(toml::from_str::<TuiSettings>(input).is_err(), "{input}");
    }
}

#[test]
fn color_parser_handles_rgb_ansi_terminal_and_unicode_safely() {
    use super::colors::ThemeColor;
    assert_eq!(
        ThemeColor::parse("#AbCDef").unwrap().0,
        Color::Rgb(171, 205, 239)
    );
    assert_eq!(ThemeColor::parse("light-blue").unwrap().0, Color::LightBlue);
    assert_eq!(ThemeColor::parse("terminal").unwrap().0, Color::Reset);
    for value in ["#é1234", "#1234567", "#gggggg", "rgb(1,2,3)", ""] {
        assert!(ThemeColor::parse(value).is_err());
    }
}

#[test]
fn lists_builtins_before_sorted_custom_names() {
    let settings: TuiSettings = toml::from_str("[themes.zebra]\n[themes.alpha]").unwrap();
    let names = settings.theme_names();
    assert_eq!(&names[..8], TuiTheme::CANONICAL_NAMES);
    assert_eq!(&names[8..], ["alpha", "zebra"]);
}

#[test]
fn border_style_defaults_to_plain_and_parses_rounded() {
    let settings: TuiSettings = toml::from_str("").unwrap();
    assert_eq!(settings.border_style, TuiBorderStyle::Plain);

    let settings: TuiSettings = toml::from_str("border_style = 'rounded'").unwrap();
    assert_eq!(settings.border_style, TuiBorderStyle::Rounded);

    assert!(toml::from_str::<TuiSettings>("border_style = 'double'").is_err());
}

#[test]
fn transparent_defaults_to_false_and_overrides_background_to_reset() {
    let settings: TuiSettings = toml::from_str("").unwrap();
    assert!(!settings.transparent);
    assert_ne!(
        settings.resolve(&settings.theme).unwrap().background,
        Color::Reset
    );

    let settings: TuiSettings = toml::from_str("theme = 'catppuccin'\ntransparent = true").unwrap();
    assert!(settings.transparent);
    let palette = settings.resolve(&settings.theme).unwrap();
    assert_eq!(palette.background, Color::Reset);
    assert_eq!(palette.accent, TuiTheme::Catppuccin.palette().accent);

    let ansi_opaque: TuiSettings = toml::from_str("theme = 'ansi'").unwrap();
    assert_eq!(
        ansi_opaque.resolve(&ansi_opaque.theme).unwrap().background,
        Color::Black
    );
    let ansi_transparent: TuiSettings =
        toml::from_str("theme = 'ansi'\ntransparent = true").unwrap();
    assert_eq!(
        ansi_transparent
            .resolve(&ansi_transparent.theme)
            .unwrap()
            .background,
        Color::Reset
    );

    let serialized = toml::to_string(&settings).unwrap();
    assert_eq!(
        toml::from_str::<TuiSettings>(&serialized).unwrap(),
        settings
    );
}
