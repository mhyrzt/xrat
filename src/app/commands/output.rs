use std::io::{self, BufRead, IsTerminal, Write};

use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Align {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Style {
    Dim,
    Green,
    Red,
    Yellow,
    Cyan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Column {
    pub(crate) header: &'static str,
    pub(crate) align: Align,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Cell {
    text: String,
    style: Option<Style>,
}

impl Cell {
    pub(crate) fn plain(value: impl Into<String>) -> Self {
        Self {
            text: value.into(),
            style: None,
        }
    }

    pub(crate) fn styled(value: impl Into<String>, style: Style) -> Self {
        Self {
            text: value.into(),
            style: Some(style),
        }
    }
}

pub(crate) fn color_enabled() -> bool {
    std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

pub(crate) fn format_table(columns: &[Column], rows: &[Vec<Cell>], color: bool) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let mut widths = columns
        .iter()
        .map(|column| column.header.width())
        .collect::<Vec<_>>();

    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(index) {
                *width = (*width).max(cell.text.width());
            }
        }
    }

    let mut lines = Vec::with_capacity(rows.len() + 1);
    let header = columns
        .iter()
        .enumerate()
        .map(|(index, column)| pad(column.header, widths[index], column.align))
        .collect::<Vec<_>>()
        .join("  ");
    lines.push(apply_style(&header, Some(Style::Dim), color));

    for row in rows {
        let line = row
            .iter()
            .enumerate()
            .map(|(index, cell)| {
                let align = columns
                    .get(index)
                    .map(|column| column.align)
                    .unwrap_or(Align::Left);
                let width = widths.get(index).copied().unwrap_or_default();
                apply_style(&pad(&cell.text, width, align), cell.style, color)
            })
            .collect::<Vec<_>>()
            .join("  ");
        lines.push(line.trim_end().to_string());
    }

    lines.join("\n")
}

pub(crate) fn format_kv(title: Option<&str>, rows: &[(&str, String)], color: bool) -> String {
    let width = rows
        .iter()
        .map(|(label, _)| label.width())
        .max()
        .unwrap_or_default();
    let mut lines = Vec::with_capacity(rows.len() + usize::from(title.is_some()));

    if let Some(title) = title {
        lines.push(apply_style(title, Some(Style::Dim), color));
    }

    for (label, value) in rows {
        lines.push(format!(
            "{}  {}",
            apply_style(&pad(label, width, Align::Left), Some(Style::Dim), color),
            value
        ));
    }

    lines.join("\n")
}

pub(crate) fn format_list(title: &str, items: &[String], color: bool) -> String {
    let mut lines = Vec::with_capacity(items.len() + 1);
    lines.push(apply_style(title, Some(Style::Dim), color));
    for item in items {
        lines.push(format!("  {item}"));
    }
    lines.join("\n")
}

pub(crate) fn success(message: impl AsRef<str>, color: bool) -> String {
    format!(
        "{} {}",
        apply_style("OK", Some(Style::Green), color),
        message.as_ref()
    )
}

pub(crate) fn notice(message: impl AsRef<str>, color: bool) -> String {
    format!(
        "{} {}",
        apply_style("INFO", Some(Style::Cyan), color),
        message.as_ref()
    )
}

pub(crate) fn warn(message: impl AsRef<str>, color: bool) -> String {
    format!(
        "{} {}",
        apply_style("WARN", Some(Style::Yellow), color),
        message.as_ref()
    )
}

pub(crate) fn empty_message(message: impl AsRef<str>) -> String {
    message.as_ref().to_string()
}

/// Prompt for a destructive confirmation. Defaults to No; a non-interactive
/// stdin (no TTY) is treated as a denial so scripts must pass an explicit flag.
pub(crate) fn confirm(prompt: impl AsRef<str>) -> io::Result<bool> {
    if !std::io::stdin().is_terminal() {
        return Ok(false);
    }

    print!("{} [y/N] ", prompt.as_ref());
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().lock().read_line(&mut answer)?;
    let answer = answer.trim().to_ascii_lowercase();
    Ok(answer == "y" || answer == "yes")
}

pub(crate) fn dash(value: Option<&str>) -> String {
    value
        .filter(|value| !value.is_empty())
        .unwrap_or("-")
        .to_string()
}

pub(crate) fn bool_label(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

pub(crate) fn truncate(text: &str, max_width: usize) -> String {
    if text.width() <= max_width {
        return text.to_string();
    }

    let mut result = String::new();
    let mut used = 0usize;
    for ch in text.chars() {
        let char_width = ch.to_string().width();
        if used + char_width > max_width.saturating_sub(1) {
            break;
        }
        result.push(ch);
        used += char_width;
    }
    result.push('…');
    result
}

fn pad(text: &str, width: usize, align: Align) -> String {
    let padding = width.saturating_sub(text.width());
    let spaces = " ".repeat(padding);
    match align {
        Align::Left => format!("{text}{spaces}"),
        Align::Right => format!("{spaces}{text}"),
    }
}

pub(crate) fn style_text(text: &str, style: Style, color: bool) -> String {
    apply_style(text, Some(style), color)
}

fn apply_style(text: &str, style: Option<Style>, color: bool) -> String {
    if !color {
        return text.to_string();
    }

    let Some(style) = style else {
        return text.to_string();
    };
    let code = match style {
        Style::Dim => "\x1b[2m",
        Style::Green => "\x1b[32m",
        Style::Red => "\x1b[31m",
        Style::Yellow => "\x1b[33m",
        Style::Cyan => "\x1b[36m",
    };
    format!("{code}{text}\x1b[0m")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_unicode_width_table_without_color() {
        let table = format_table(
            &[
                Column {
                    header: "ID",
                    align: Align::Right,
                },
                Column {
                    header: "NAME",
                    align: Align::Left,
                },
            ],
            &[vec![Cell::plain("7"), Cell::plain("节点")]],
            false,
        );

        assert_eq!(table, "ID  NAME\n 7  节点");
    }

    #[test]
    fn truncates_by_display_width() {
        assert_eq!(truncate("abcdef", 4), "abc…");
        assert_eq!(truncate("节点一", 5), "节点…");
    }
}
