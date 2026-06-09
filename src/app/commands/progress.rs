use std::io::IsTerminal;
use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub(crate) struct CliProgress {
    bar: Option<ProgressBar>,
}

impl CliProgress {
    pub(crate) fn spinner(enabled: bool, message: impl Into<String>) -> Self {
        if !should_enable(enabled, std::io::stderr().is_terminal()) {
            return Self::disabled();
        }

        let bar = ProgressBar::new_spinner();
        let style = ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner());
        bar.set_style(style);
        bar.set_message(message.into());
        bar.enable_steady_tick(Duration::from_millis(100));
        Self { bar: Some(bar) }
    }

    pub(crate) fn bar(enabled: bool, len: u64, message: impl Into<String>) -> Self {
        Self::bar_with_template(
            enabled,
            len,
            message,
            "{spinner:.green} {msg} [{bar:32.cyan/blue}] {pos}/{len}",
        )
    }

    pub(crate) fn bytes_bar(
        enabled: bool,
        content_length: Option<u64>,
        message: impl Into<String>,
    ) -> Self {
        Self::bar_with_template(
            enabled,
            content_length.unwrap_or(0),
            message,
            "{spinner:.green} {msg} [{bar:32.cyan/blue}] {bytes}/{total_bytes}",
        )
    }

    pub(crate) fn bar_with_template(
        enabled: bool,
        len: u64,
        message: impl Into<String>,
        template: &'static str,
    ) -> Self {
        if !should_enable(enabled, std::io::stderr().is_terminal()) {
            return Self::disabled();
        }

        let bar = ProgressBar::new(len);
        let style = ProgressStyle::with_template(template)
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("=>-");
        bar.set_style(style);
        bar.set_message(message.into());
        Self { bar: Some(bar) }
    }

    pub(crate) fn bytes_bar_in_multi(
        enabled: bool,
        multi: &MultiProgress,
        content_length: Option<u64>,
        message: impl Into<String>,
    ) -> Self {
        if !should_enable(enabled, std::io::stderr().is_terminal()) {
            return Self::disabled();
        }

        let bar = multi.add(ProgressBar::new(content_length.unwrap_or(0)));
        let style = ProgressStyle::with_template(
            "{spinner:.green} {msg} [{bar:32.cyan/blue}] {bytes}/{total_bytes}",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar())
        .progress_chars("=>-");
        bar.set_style(style);
        bar.set_message(message.into());
        Self { bar: Some(bar) }
    }

    pub(crate) fn disabled() -> Self {
        Self { bar: None }
    }

    pub(crate) fn inc(&self, delta: u64) {
        if let Some(bar) = &self.bar {
            bar.inc(delta);
        }
    }

    pub(crate) fn set_position(&self, position: u64) {
        if let Some(bar) = &self.bar {
            bar.set_position(position);
        }
    }

    pub(crate) fn set_length(&self, len: u64) {
        if let Some(bar) = &self.bar {
            bar.set_length(len);
        }
    }

    pub(crate) fn set_message(&self, message: impl Into<String>) {
        if let Some(bar) = &self.bar {
            bar.set_message(message.into());
        }
    }

    pub(crate) fn finish_with_message(self, message: impl Into<String>) {
        if let Some(bar) = self.bar {
            bar.finish_with_message(message.into());
        }
    }

    pub(crate) fn finish_and_clear(self) {
        if let Some(bar) = self.bar {
            bar.finish_and_clear();
        }
    }

    pub(crate) fn abandon_with_message(&self, message: impl Into<String>) {
        if let Some(bar) = &self.bar {
            bar.abandon_with_message(message.into());
        }
    }
}

pub(crate) fn should_enable(enabled: bool, stderr_is_terminal: bool) -> bool {
    enabled && stderr_is_terminal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_requires_enabled_flag_and_tty_stderr() {
        assert!(should_enable(true, true));
        assert!(!should_enable(false, true));
        assert!(!should_enable(true, false));
        assert!(!should_enable(false, false));
    }
}
