//! Per-step status model shared by the apply flow and `--check` diagnostics.

use serde::Serialize;

use crate::app::commands::output::{self, Align, Cell, Column, Style};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    /// The step performed work this run.
    Done,
    /// The step was already satisfied; nothing changed.
    AlreadyDone,
    /// The step was intentionally not run (flag, prompt, or platform).
    Skipped,
    /// A required or expected piece is missing (reported by `--check`).
    Missing,
    /// The step attempted work and failed.
    Failed,
}

impl StepStatus {
    fn label(self) -> &'static str {
        match self {
            StepStatus::Done => "done",
            StepStatus::AlreadyDone => "already done",
            StepStatus::Skipped => "skipped",
            StepStatus::Missing => "missing",
            StepStatus::Failed => "failed",
        }
    }

    fn style(self) -> Style {
        match self {
            StepStatus::Done => Style::Green,
            StepStatus::AlreadyDone => Style::Dim,
            StepStatus::Skipped => Style::Dim,
            StepStatus::Missing => Style::Yellow,
            StepStatus::Failed => Style::Red,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct StepOutcome {
    pub name: &'static str,
    pub status: StepStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    pub required: bool,
}

impl StepOutcome {
    pub fn new(name: &'static str, status: StepStatus, required: bool) -> Self {
        Self {
            name,
            status,
            detail: None,
            required,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

/// True when any required step is missing or failed (drives a non-zero exit).
pub fn has_blocking_failure(outcomes: &[StepOutcome]) -> bool {
    outcomes.iter().any(|outcome| {
        outcome.required && matches!(outcome.status, StepStatus::Missing | StepStatus::Failed)
    })
}

pub fn render_table(outcomes: &[StepOutcome]) -> String {
    let color = output::color_enabled();
    let columns = [
        Column {
            header: "STEP",
            align: Align::Left,
        },
        Column {
            header: "STATUS",
            align: Align::Left,
        },
        Column {
            header: "DETAIL",
            align: Align::Left,
        },
    ];
    let rows = outcomes
        .iter()
        .map(|outcome| {
            vec![
                Cell::plain(outcome.name),
                Cell::styled(outcome.status.label(), outcome.status.style()),
                Cell::plain(outcome.detail.clone().unwrap_or_else(|| "-".to_string())),
            ]
        })
        .collect::<Vec<_>>();
    output::format_table(&columns, &rows, color)
}

pub fn render_json(outcomes: &[StepOutcome]) -> crate::app::Result<String> {
    Ok(serde_json::to_string_pretty(outcomes)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<StepOutcome> {
        vec![
            StepOutcome::new("xray", StepStatus::Done, true).with_detail("/usr/bin/xray"),
            StepOutcome::new("daemon", StepStatus::Missing, false),
        ]
    }

    #[test]
    fn json_includes_status_and_detail() {
        let json = render_json(&sample()).unwrap();
        assert!(json.contains("\"name\": \"xray\""));
        assert!(json.contains("\"status\": \"done\""));
        assert!(json.contains("\"detail\": \"/usr/bin/xray\""));
        assert!(json.contains("\"status\": \"missing\""));
    }

    #[test]
    fn json_omits_absent_detail() {
        let json = render_json(&[StepOutcome::new("daemon", StepStatus::Missing, false)]).unwrap();
        assert!(!json.contains("\"detail\""));
    }

    #[test]
    fn table_lists_steps_and_status_words() {
        let table = render_table(&sample());
        assert!(table.contains("xray"));
        assert!(table.contains("daemon"));
        assert!(table.contains("done"));
        assert!(table.contains("missing"));
    }

    #[test]
    fn blocking_failure_only_for_required_steps() {
        assert!(has_blocking_failure(&[StepOutcome::new(
            "xray",
            StepStatus::Missing,
            true
        )]));
        assert!(!has_blocking_failure(&[StepOutcome::new(
            "daemon",
            StepStatus::Missing,
            false
        )]));
        assert!(!has_blocking_failure(&sample()));
    }
}
