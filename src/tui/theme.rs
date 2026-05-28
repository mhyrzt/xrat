use ratatui::style::{Color, Style};

pub fn chrome_style() -> Style {
    Style::default().fg(Color::Rgb(237, 225, 205))
}

pub fn accent_style() -> Style {
    Style::default().fg(Color::Rgb(247, 181, 99))
}

pub fn muted_style() -> Style {
    Style::default().fg(Color::Rgb(148, 139, 123))
}

pub fn success_style() -> Style {
    Style::default().fg(Color::Rgb(97, 198, 128))
}

pub fn failure_style() -> Style {
    Style::default().fg(Color::Rgb(229, 96, 84))
}
