use super::diagnostic::Diagnostic;
use crate::app::config::{CustomTheme, ThemeColor, ThemeName, TuiSettings};
use crate::tui::theme::TuiTheme;

pub(super) fn check_fields(value: &toml::Value, errors: &mut Vec<Diagnostic>) {
    for section in ["tui", "ui"] {
        let Some(settings) = value.get(section) else {
            continue;
        };
        let before = errors.len();
        if let Some(table) = settings.as_table() {
            for field in table.keys().filter(|field| {
                !matches!(
                    field.as_str(),
                    "theme" | "themes" | "border_style" | "transparent"
                )
            }) {
                invalid(
                    errors,
                    format!("[{section}].{field}"),
                    "unknown TUI setting".to_string(),
                );
            }
            if let Some(border_style) = table.get("border_style") {
                if let Some(style) = border_style.as_str() {
                    if let Err(error) = crate::app::config::TuiBorderStyle::parse(style) {
                        invalid(errors, format!("[{section}].border_style"), error);
                    }
                } else {
                    invalid(
                        errors,
                        format!("[{section}].border_style"),
                        "expected a border style string (plain or rounded)".to_string(),
                    );
                }
            }
            if let Some(transparent) = table.get("transparent")
                && !transparent.is_bool()
            {
                invalid(
                    errors,
                    format!("[{section}].transparent"),
                    "expected a boolean (true or false)".to_string(),
                );
            }
            if let Some(theme) = table.get("theme") {
                if let Some(name) = theme.as_str() {
                    if let Err(error) = ThemeName::parse(name) {
                        invalid(errors, format!("[{section}].theme"), error);
                    }
                } else {
                    invalid(
                        errors,
                        format!("[{section}].theme"),
                        "expected a theme name string".to_string(),
                    );
                }
            }
            if let Some(themes) = table.get("themes").and_then(toml::Value::as_table) {
                for (name, theme) in themes {
                    let prefix = format!("[{section}.themes.{name}]");
                    if !matches!(ThemeName::parse(name), Ok(ThemeName::Custom(_))) {
                        invalid(errors, prefix.clone(), "invalid or reserved custom name; built-in names and aliases are reserved".to_string());
                    }
                    let Some(fields) = theme.as_table() else {
                        invalid(errors, prefix, "expected a custom theme table".to_string());
                        continue;
                    };
                    for (field, value) in fields {
                        let result = if field == "extends" {
                            value
                                .as_str()
                                .ok_or_else(|| "expected a built-in theme name string".to_string())
                                .and_then(|name| name.parse::<TuiTheme>().map(|_| ()))
                        } else if CustomTheme::COLOR_FIELDS.contains(&field.as_str()) {
                            value
                                .as_str()
                                .ok_or_else(|| "expected a color string".to_string())
                                .and_then(|color| ThemeColor::parse(color).map(|_| ()))
                        } else {
                            Err(format!("unknown color field {field:?}"))
                        };
                        if let Err(error) = result {
                            invalid(errors, format!("{prefix}.{field}"), error);
                        }
                    }
                }
            }
        }
        if before != errors.len() {
            continue;
        }
        let parsed = toml::to_string(settings)
            .map_err(|error| error.to_string())
            .and_then(|text| {
                toml::from_str::<TuiSettings>(&text).map_err(|error| error.to_string())
            });
        if let Err(error) = parsed {
            let field = if error.contains("tui.theme:") {
                format!("[{section}].theme")
            } else {
                format!("[{section}]")
            };
            invalid(errors, field, error);
        }
    }
}

fn invalid(errors: &mut Vec<Diagnostic>, field: String, problem: String) {
    errors.push(Diagnostic::new(field, problem,
        "theme names and color overrides must resolve to a valid palette.",
        format!("choose one of {} or define a named custom theme using known fields and #RRGGBB, ANSI names or terminal colors.", TuiTheme::CANONICAL_NAMES.join(", "))));
}
