use std::io::IsTerminal;

use unicode_width::UnicodeWidthStr;

use super::*;
use crate::support::refs::short_ref;

pub(crate) fn write_results(args: &TestArgs, outputs: &[TestOutputRow]) -> crate::app::Result<()> {
    let data = match args.format {
        TestFormat::Table => {
            let color = args.output.is_none() && std::io::stdout().is_terminal();
            format_table(outputs, color)
        }
        TestFormat::Tsv => format_tsv(outputs),
        TestFormat::Csv => format_csv(outputs),
        TestFormat::Json => serde_json::to_string_pretty(outputs)?,
    };

    if let Some(path) = &args.output {
        std::fs::write(path, data)?;
    } else {
        println!("{data}");
    }

    Ok(())
}

const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";
const MAX_NAME_WIDTH: usize = 28;

pub(crate) fn format_table(outputs: &[TestOutputRow], color: bool) -> String {
    if outputs.is_empty() {
        return "No configs matched.".to_string();
    }

    let metric_columns = TestMetricColumns::from_outputs(outputs);
    let mut headers = vec!["REF", "STATUS"];
    metric_columns.push_headers(&mut headers);
    headers.extend(["PROTO", "ADDRESS", "PORT", "NAME"]);
    let right_aligned = headers
        .iter()
        .map(|header| matches!(*header, "ICMP" | "REAL" | "DOWN" | "UP" | "PORT"))
        .collect::<Vec<_>>();

    let mut rows: Vec<Vec<String>> = Vec::with_capacity(outputs.len());
    for output in outputs {
        let mut row = vec![
            short_ref(&output.r#ref).to_string(),
            status_label(output.status),
        ];
        metric_columns.push_table_cells(output, &mut row);
        row.extend([
            output.protocol.clone(),
            dash_cell(&output.address),
            output.port.to_string(),
            truncate_display(output.name.as_deref().unwrap_or("-"), MAX_NAME_WIDTH),
        ]);
        rows.push(row);
    }

    let mut widths = vec![0usize; headers.len()];
    for (index, header) in headers.iter().enumerate() {
        widths[index] = header.width();
    }
    for row in &rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(cell.width());
        }
    }

    let mut lines = Vec::with_capacity(rows.len() + 2);

    let header_line = headers
        .iter()
        .enumerate()
        .map(|(index, header)| pad(header, widths[index], right_aligned[index]))
        .collect::<Vec<_>>()
        .join("  ");
    lines.push(maybe_color(&header_line, DIM, color));

    for (output, row) in outputs.iter().zip(&rows) {
        let cells = row
            .iter()
            .enumerate()
            .map(|(index, cell)| {
                let padded = pad(cell, widths[index], right_aligned[index]);
                if index == 1 && color {
                    colorize_status(&padded, output.status)
                } else {
                    padded
                }
            })
            .collect::<Vec<_>>()
            .join("  ");
        lines.push(cells.trim_end().to_string());
    }

    let failures: Vec<&TestOutputRow> = outputs
        .iter()
        .filter(|output| matches!(output.status, TestStatus::Failed) && output.error.is_some())
        .collect();
    if !failures.is_empty() {
        lines.push(String::new());
        lines.push(maybe_color("Failures:", DIM, color));
        for output in failures {
            let reason = output.error.as_deref().unwrap_or_default();
            lines.push(format!("  {} {}", short_ref(&output.r#ref), reason));
        }
    }

    lines.join("\n")
}

fn status_label(status: TestStatus) -> String {
    match status {
        TestStatus::Ok => "✓ ok".to_string(),
        TestStatus::Failed => "✗ fail".to_string(),
        TestStatus::Skipped => "– skip".to_string(),
    }
}

fn colorize_status(text: &str, status: TestStatus) -> String {
    let code = match status {
        TestStatus::Ok => GREEN,
        TestStatus::Failed => RED,
        TestStatus::Skipped => DIM,
    };
    format!("{code}{text}{RESET}")
}

fn maybe_color(text: &str, code: &str, color: bool) -> String {
    if color {
        format!("{code}{text}{RESET}")
    } else {
        text.to_string()
    }
}

fn dash_cell(value: &str) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        value.to_string()
    }
}

fn ms_cell(value: Option<u32>) -> String {
    value
        .map(|value| format!("{value}ms"))
        .unwrap_or_else(|| "-".to_string())
}

fn mbps_cell(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.1}"))
        .unwrap_or_else(|| "-".to_string())
}

fn pad(text: &str, width: usize, right_aligned: bool) -> String {
    let padding = width.saturating_sub(text.width());
    let spaces = " ".repeat(padding);
    if right_aligned {
        format!("{spaces}{text}")
    } else {
        format!("{text}{spaces}")
    }
}

fn truncate_display(text: &str, max_width: usize) -> String {
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

pub(crate) fn format_tsv(outputs: &[TestOutputRow]) -> String {
    let metric_columns = TestMetricColumns::from_outputs(outputs);
    let mut lines = Vec::with_capacity(outputs.len() + 1);
    let mut headers = vec!["ref", "name", "protocol", "address", "port"];
    metric_columns.push_raw_headers(&mut headers);
    headers.extend(["status", "error"]);
    lines.push(headers.join("\t"));

    for output in outputs {
        let mut cells = vec![
            output.r#ref.clone(),
            tsv_cell(output.name.as_deref()),
            output.protocol.clone(),
            output.address.clone(),
            output.port.to_string(),
        ];
        metric_columns.push_raw_cells(output, &mut cells);
        cells.extend([
            output.status.as_str().to_string(),
            tsv_cell(output.error.as_deref()),
        ]);
        lines.push(cells.join("\t"));
    }

    lines.join("\n")
}

pub(crate) fn format_csv(outputs: &[TestOutputRow]) -> String {
    let metric_columns = TestMetricColumns::from_outputs(outputs);
    let mut lines = Vec::with_capacity(outputs.len() + 1);
    let mut headers = vec!["ref", "name", "protocol", "address", "port"];
    metric_columns.push_raw_headers(&mut headers);
    headers.extend(["status", "error"]);
    lines.push(headers.join(","));

    for output in outputs {
        let mut cells = vec![
            csv_cell(Some(&output.r#ref)),
            csv_cell(output.name.as_deref()),
            csv_cell(Some(&output.protocol)),
            csv_cell(Some(&output.address)),
            output.port.to_string(),
        ];
        metric_columns.push_raw_cells(output, &mut cells);
        cells.extend([
            output.status.as_str().to_string(),
            csv_cell(output.error.as_deref()),
        ]);
        lines.push(cells.join(","));
    }

    lines.join("\n")
}

fn tsv_cell(value: Option<&str>) -> String {
    value.unwrap_or_default().replace(['\t', '\r', '\n'], " ")
}

fn csv_cell(value: Option<&str>) -> String {
    let value = value.unwrap_or_default();
    if value.contains(&[',', '"', '\r', '\n'][..]) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub(crate) fn optional_number(value: Option<u32>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

pub(crate) fn optional_float(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.2}")).unwrap_or_default()
}

#[derive(Clone, Copy, Debug, Default)]
struct TestMetricColumns {
    icmp: bool,
    real_delay: bool,
    download: bool,
    upload: bool,
}

impl TestMetricColumns {
    fn from_outputs(outputs: &[TestOutputRow]) -> Self {
        Self {
            icmp: outputs.iter().any(|output| output.ran_icmp),
            real_delay: outputs.iter().any(|output| output.ran_real_delay),
            download: outputs.iter().any(|output| output.download_mbps.is_some()),
            upload: outputs.iter().any(|output| output.upload_mbps.is_some()),
        }
    }

    fn push_headers(self, headers: &mut Vec<&'static str>) {
        if self.icmp {
            headers.push("ICMP");
        }
        if self.real_delay {
            headers.push("REAL");
        }
        if self.download {
            headers.push("DOWN");
        }
        if self.upload {
            headers.push("UP");
        }
    }

    fn push_raw_headers(self, headers: &mut Vec<&'static str>) {
        if self.icmp {
            headers.push("icmp_ms");
        }
        if self.real_delay {
            headers.push("real_delay_ms");
        }
        if self.download {
            headers.push("download_mbps");
        }
        if self.upload {
            headers.push("upload_mbps");
        }
    }

    fn push_table_cells(self, output: &TestOutputRow, cells: &mut Vec<String>) {
        if self.icmp {
            cells.push(ms_cell(output.icmp_ms));
        }
        if self.real_delay {
            cells.push(ms_cell(output.real_delay_ms));
        }
        if self.download {
            cells.push(mbps_cell(output.download_mbps));
        }
        if self.upload {
            cells.push(mbps_cell(output.upload_mbps));
        }
    }

    fn push_raw_cells(self, output: &TestOutputRow, cells: &mut Vec<String>) {
        if self.icmp {
            cells.push(optional_number(output.icmp_ms));
        }
        if self.real_delay {
            cells.push(optional_number(output.real_delay_ms));
        }
        if self.download {
            cells.push(optional_float(output.download_mbps));
        }
        if self.upload {
            cells.push(optional_float(output.upload_mbps));
        }
    }
}
