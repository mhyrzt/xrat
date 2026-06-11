//! Traffic tab dashboard. Unlike the other log tabs (which share the scrolling
//! line panel), the Traffic tab draws real widgets:
//!
//! - Row 1 (compact): a throughput table and a per-profile probe table.
//! - Row 2 (the rest): a bidirectional traffic chart (upload up, download down,
//!   independently scaled) with timestamped loss markers, beside a probe graph
//!   plotting each active latency metric as its own colored line.
//!
//! Inner cards are borderless; only the graphs carry a short title.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Axis, Block, Borders, Cell, Chart, Clear, Dataset, GraphType, Paragraph, Row, Table,
};

use crate::tui::app::TuiApp;
use crate::tui::data::MetricSummary;
use crate::tui::theme;

pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &TuiApp,
    title: Line<'static>,
    focused: bool,
) {
    let border_style = if focused {
        theme::accent_style()
    } else {
        theme::chrome_style()
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    if !app.data.runtime.pid_running {
        render_message(
            frame,
            inner,
            "Runtime is not active; no live traffic stats.",
        );
        return;
    }
    let Some(latest) = app.stats.latest() else {
        render_message(frame, inner, "Waiting for traffic stats from the engine…");
        return;
    };

    // Keep row 1 compact (header + active metric rows, target three) so the
    // graphs own most of the height.
    let text_height = (active_metric_count(app) as u16 + 1).clamp(3, 6);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .spacing(1)
        .constraints([Constraint::Length(text_height), Constraint::Min(0)])
        .split(inner);
    let text_row = Layout::default()
        .direction(Direction::Horizontal)
        .spacing(2)
        .constraints([Constraint::Percentage(32), Constraint::Percentage(68)])
        .split(rows[0]);
    let graph_row = Layout::default()
        .direction(Direction::Horizontal)
        .spacing(2)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(rows[1]);

    render_traffic_summary(
        frame,
        text_row[0],
        latest.downlink_total,
        latest.uplink_total,
        latest.down_rate,
        latest.up_rate,
    );
    render_probe_table(frame, text_row[1], app);
    render_traffic_graph(frame, graph_row[0], app);
    render_probe_graph(frame, graph_row[1], app);
}

fn active_metric_count(app: &TuiApp) -> usize {
    let columns = app.data.metric_columns;
    [
        columns.icmp,
        columns.tcp,
        columns.real_delay,
        columns.download,
        columns.upload,
    ]
    .iter()
    .filter(|active| **active)
    .count()
}

// ---- row 1: text tables ---------------------------------------------------

fn render_traffic_summary(
    frame: &mut Frame<'_>,
    area: Rect,
    down_total: u64,
    up_total: u64,
    down_rate: u64,
    up_rate: u64,
) {
    let lines = vec![
        traffic_line("total", &format_bytes(down_total), &format_bytes(up_total)),
        traffic_line(
            "rate",
            &format!("{}/s", format_bytes(down_rate)),
            &format!("{}/s", format_bytes(up_rate)),
        ),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

/// `label  ↓ <down>  ↑ <up>` row for totals and throughput.
fn traffic_line(label: &str, down: &str, up: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<6}"), theme::muted_style()),
        Span::styled("↓ ", color_for("download")),
        Span::styled(format!("{down:<11}"), theme::chrome_style()),
        Span::styled("↑ ", color_for("upload")),
        Span::styled(up.to_string(), theme::chrome_style()),
    ])
}

fn render_probe_table(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let history = &app.data.probe_history;
    let columns = app.data.metric_columns;
    let last = history
        .last_tested
        .as_deref()
        .map(short_time)
        .unwrap_or_else(|| "-".to_string());

    let mut rows = Vec::new();
    let mut push =
        |active: bool, label: &str, summary: &MetricSummary, unit: &str, decimals: usize| {
            if active {
                rows.push(metric_row(label, summary, unit, decimals, &last));
            }
        };
    push(columns.icmp, "icmp", &history.icmp, "ms", 0);
    push(columns.tcp, "tcp", &history.tcp, "ms", 0);
    push(
        columns.real_delay,
        "real-delay",
        &history.real_delay,
        "ms",
        0,
    );
    push(columns.download, "download", &history.download, "Mbps", 1);
    push(columns.upload, "upload", &history.upload, "Mbps", 1);

    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::styled(
                "No probe profiles active.",
                theme::muted_style(),
            )),
            area,
        );
        return;
    }

    let header =
        Row::new(["Name", "Value", "mean ± std", "n", "last update"]).style(theme::muted_style());
    let widths = [
        Constraint::Length(11),
        Constraint::Length(11),
        Constraint::Length(16),
        Constraint::Length(4),
        Constraint::Length(10),
    ];
    let table = Table::new(rows, widths).header(header);
    frame.render_widget(table, area);
}

fn metric_row(
    label: &str,
    summary: &MetricSummary,
    unit: &str,
    decimals: usize,
    last: &str,
) -> Row<'static> {
    let value = match summary.current {
        Some(current) => format!("{} {unit}", format_num(current, decimals)),
        None => "-".to_string(),
    };
    let mean_std = match (summary.mean, summary.std) {
        (Some(mean), Some(std)) if summary.count > 1 => format!(
            "{} ± {}",
            format_num(mean, decimals),
            format_num(std, decimals)
        ),
        _ => "-".to_string(),
    };
    Row::new(vec![
        Cell::from(Span::styled(label.to_string(), color_for(label))),
        Cell::from(value),
        Cell::from(mean_std),
        Cell::from(summary.count.to_string()),
        Cell::from(last.to_string()),
    ])
}

// ---- row 2: graphs --------------------------------------------------------

/// One latency series for the probe graph: label, point marker, and points.
type ProbeSeries<'a> = (&'a str, Marker, &'a [(f64, f64)]);

fn render_traffic_graph(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let body = titled_body(frame, area, "traffic");

    // Pack one bar per column and drop older samples so the window stays full
    // width instead of slowly filling with gaps.
    let bar_count = (body.width.saturating_sub(12)).max(8) as usize;
    let down = window_tail(&app.stats.down_rates(), bar_count);
    let up = window_tail(&app.stats.up_rates(), bar_count);

    // Upload and download are scaled independently against their own peak, then
    // mapped into the top/bottom halves of a shared [-1, 1] axis.
    let max_up = up.iter().copied().max().unwrap_or(0).max(1) as f64;
    let max_down = down.iter().copied().max().unwrap_or(0).max(1) as f64;
    let len = down.len().max(up.len());
    let x_bound = len.saturating_sub(1).max(1) as f64;

    let up_points: Vec<(f64, f64)> = up
        .iter()
        .enumerate()
        .map(|(index, &value)| (index as f64, value as f64 / max_up))
        .collect();
    // Download grows downward: negate so its bars sit below the zero axis.
    let down_points: Vec<(f64, f64)> = down
        .iter()
        .enumerate()
        .map(|(index, &value)| (index as f64, -(value as f64 / max_down)))
        .collect();

    // Map recent failures onto the time window: the rightmost bar is "now", and
    // each bar is one poll (~1s), so a failure k seconds ago sits k bars left.
    let failure_points: Vec<(f64, f64)> = app
        .data
        .probe_history
        .failure_secs_ago
        .iter()
        .filter_map(|&secs_ago| {
            let x = x_bound - secs_ago as f64;
            (x >= 0.0).then_some((x, 0.0))
        })
        .collect();

    let mut datasets = vec![
        Dataset::default()
            .data(&up_points)
            .graph_type(GraphType::Bar)
            .marker(Marker::HalfBlock)
            .style(color_for("upload")),
        Dataset::default()
            .data(&down_points)
            .graph_type(GraphType::Bar)
            .marker(Marker::HalfBlock)
            .style(color_for("download")),
    ];
    if !failure_points.is_empty() {
        datasets.push(
            Dataset::default()
                .name("loss")
                .data(&failure_points)
                .graph_type(GraphType::Scatter)
                .marker(Marker::Dot)
                .style(theme::failure_style()),
        );
    }

    let y_labels = vec![
        Line::styled(
            format!("↓ {}", format_bytes(max_down as u64)),
            color_for("download"),
        ),
        Line::styled(
            format!("↓ {}", format_bytes((max_down / 2.0) as u64)),
            theme::muted_style(),
        ),
        Line::styled("0", theme::muted_style()),
        Line::styled(
            format!("↑ {}", format_bytes((max_up / 2.0) as u64)),
            theme::muted_style(),
        ),
        Line::styled(
            format!("↑ {}", format_bytes(max_up as u64)),
            color_for("upload"),
        ),
    ];

    let chart = Chart::new(datasets)
        .x_axis(Axis::default().bounds([0.0, x_bound]))
        .y_axis(
            Axis::default()
                .bounds([-1.0, 1.0])
                .labels(y_labels)
                .style(theme::muted_style()),
        );
    frame.render_widget(chart, body);
}

fn render_probe_graph(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let body = titled_body(frame, area, "probes (ms)");
    let history = &app.data.probe_history;
    let columns = app.data.metric_columns;

    let mut series: Vec<ProbeSeries> = Vec::new();
    if columns.icmp && !history.icmp_points.is_empty() {
        series.push(("icmp", Marker::Dot, &history.icmp_points));
    }
    if columns.tcp && !history.tcp_points.is_empty() {
        series.push(("tcp", Marker::Block, &history.tcp_points));
    }
    if columns.real_delay && !history.real_delay_points.is_empty() {
        series.push(("real-delay", Marker::Braille, &history.real_delay_points));
    }

    if series.is_empty() {
        render_message(frame, body, "No probe history yet.");
        return;
    }

    let max_y = series
        .iter()
        .flat_map(|(_, _, points)| points.iter().map(|&(_, y)| y))
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let x_bound = history.run_count.saturating_sub(1).max(1) as f64;

    let datasets: Vec<Dataset> = series
        .iter()
        .map(|(label, marker, points)| {
            Dataset::default()
                .name(*label)
                .data(points)
                .graph_type(GraphType::Line)
                .marker(*marker)
                .style(color_for(label))
        })
        .collect();

    let y_labels = vec![
        Line::styled("0", theme::muted_style()),
        Line::styled(
            format!("{} ms", (max_y / 2.0).round() as i64),
            theme::muted_style(),
        ),
        Line::styled(format!("{} ms", max_y.round() as i64), theme::muted_style()),
    ];

    let chart = Chart::new(datasets)
        .x_axis(Axis::default().bounds([0.0, x_bound]))
        .y_axis(
            Axis::default()
                .bounds([0.0, max_y * 1.1])
                .labels(y_labels)
                .style(theme::muted_style()),
        );
    frame.render_widget(chart, body);
}

// ---- helpers --------------------------------------------------------------

/// Render a borderless one-row title above `area` and return the body below it.
fn titled_body(frame: &mut Frame<'_>, area: Rect, title: &str) -> Rect {
    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);
    frame.render_widget(
        Paragraph::new(Line::styled(format!(" {title}"), theme::accent_style())),
        parts[0],
    );
    parts[1]
}

fn render_message(frame: &mut Frame<'_>, area: Rect, message: &str) {
    frame.render_widget(
        Paragraph::new(Line::styled(message.to_string(), theme::muted_style())),
        area,
    );
}

/// Fixed per-metric colors, shared between the probe table labels and graph
/// lines so the two read as the same series.
fn color_for(label: &str) -> Style {
    let color = match label {
        "icmp" => Color::Rgb(97, 198, 128),
        "tcp" => Color::Rgb(204, 170, 80),
        "real-delay" => Color::Rgb(247, 181, 99),
        "download" => Color::Rgb(120, 180, 235),
        "upload" => Color::Rgb(210, 130, 220),
        _ => return theme::chrome_style(),
    };
    Style::default().fg(color)
}

/// Last `count` elements of `values`, copied into a fresh vector.
fn window_tail(values: &[u64], count: usize) -> Vec<u64> {
    let start = values.len().saturating_sub(count);
    values[start..].to_vec()
}

fn short_time(value: &str) -> String {
    let normalized = value.trim_end_matches('Z').replace('T', " ");
    match normalized.split_once(' ') {
        Some((_, time)) => time.chars().take(8).collect(),
        None => normalized.chars().take(8).collect(),
    }
}

fn format_num(value: f64, decimals: usize) -> String {
    if decimals == 0 {
        format!("{}", value.round() as i64)
    } else {
        format!("{value:.decimals$}")
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
