use super::*;

#[test]
fn parses_canonical_names_and_aliases() {
    assert_eq!("default".parse::<TuiTheme>().unwrap(), TuiTheme::Default);
    assert_eq!("amber".parse::<TuiTheme>().unwrap(), TuiTheme::Default);
    assert_eq!("warm".parse::<TuiTheme>().unwrap(), TuiTheme::Default);
    assert_eq!("ansi".parse::<TuiTheme>().unwrap(), TuiTheme::Ansi);
    assert_eq!("terminal".parse::<TuiTheme>().unwrap(), TuiTheme::Ansi);
    assert_eq!(
        "catppuccin".parse::<TuiTheme>().unwrap(),
        TuiTheme::Catppuccin
    );
    assert_eq!("mocha".parse::<TuiTheme>().unwrap(), TuiTheme::Catppuccin);
    assert_eq!("dracula".parse::<TuiTheme>().unwrap(), TuiTheme::Dracula);
    assert_eq!("gruvbox".parse::<TuiTheme>().unwrap(), TuiTheme::Gruvbox);
    assert_eq!("nord".parse::<TuiTheme>().unwrap(), TuiTheme::Nord);
    assert_eq!(
        "tokyo-night".parse::<TuiTheme>().unwrap(),
        TuiTheme::TokyoNight
    );
    assert_eq!(
        "tokyonight".parse::<TuiTheme>().unwrap(),
        TuiTheme::TokyoNight
    );
    assert_eq!("light".parse::<TuiTheme>().unwrap(), TuiTheme::Light);
    assert_eq!("latte".parse::<TuiTheme>().unwrap(), TuiTheme::Light);
}

#[test]
fn rejects_unknown_theme() {
    assert!("invalid-theme".parse::<TuiTheme>().is_err());
}

#[test]
fn switching_theme_updates_active_palette_and_styles() {
    let _guard = lock_test_theme();
    set_active(TuiTheme::Default);
    assert_eq!(palette(), TuiTheme::Default.palette());
    let default_accent = accent_style();
    let default_secondary = secondary_style();

    set_active(TuiTheme::Nord);
    assert_eq!(palette(), TuiTheme::Nord.palette());
    let nord_accent = accent_style();
    let nord_secondary = secondary_style();
    assert_ne!(default_accent, nord_accent);
    assert_ne!(default_secondary, nord_secondary);

    set_active(TuiTheme::Default);
}

#[test]
fn latency_style_thresholds() {
    let _guard = lock_test_theme();
    set_active(TuiTheme::Default);
    assert_eq!(latency_style(50), success_style());
    assert_eq!(latency_style(250), warning_style());
    assert_eq!(latency_style(600), failure_style());

    assert_eq!(latency_text_style("42 ms"), success_style());
    assert_eq!(latency_text_style("200ms"), warning_style());
    assert_eq!(latency_text_style("500 ms"), failure_style());
    assert_eq!(latency_text_style("-"), muted_style());
}

#[test]
fn semantic_styles_use_actual_state() {
    let _guard = lock_test_theme();
    set_active(TuiTheme::Default);
    assert_eq!(available_style(true), success_style());
    assert_eq!(available_style(false), muted_style());
    assert_eq!(optional_latency_style(None), muted_style());
    assert_eq!(optional_latency_style(Some(100)), success_style());
}

#[test]
fn rgb_palettes_keep_text_readable_on_normal_and_selected_surfaces() {
    fn luminance(color: Color) -> f64 {
        let Color::Rgb(red, green, blue) = color else {
            panic!("expected RGB color")
        };
        [red, green, blue]
            .into_iter()
            .zip([0.2126, 0.7152, 0.0722])
            .map(|(channel, weight)| {
                let channel = f64::from(channel) / 255.0;
                weight
                    * if channel <= 0.04045 {
                        channel / 12.92
                    } else {
                        ((channel + 0.055) / 1.055).powf(2.4)
                    }
            })
            .sum()
    }
    for name in TuiTheme::CANONICAL_NAMES
        .iter()
        .filter(|name| **name != "ansi")
    {
        let palette = name.parse::<TuiTheme>().unwrap().palette();
        for background in [palette.background, palette.selection_background] {
            for foreground in [
                palette.foreground,
                palette.selection_foreground,
                palette.accent,
                palette.secondary,
                palette.muted,
                palette.success,
                palette.failure,
                palette.warning,
                palette.download,
                palette.upload,
            ] {
                let bg = luminance(background);
                let fg = luminance(foreground);
                let ratio = (bg.max(fg) + 0.05) / (bg.min(fg) + 0.05);
                assert!(
                    ratio >= 4.5,
                    "{name}: {foreground:?} on {background:?}: {ratio}"
                );
            }
        }
    }
}
