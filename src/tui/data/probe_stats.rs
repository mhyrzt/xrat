//! Probe/test history for the Traffic tab's stats table and probe graph. Built
//! from the active config's recent `connection_tests` rows (newest-first, as
//! returned by `list_connection_tests`). Per-metric current value and
//! `mean ± std` come from a bounded window; latency metrics also keep
//! chronological point series for the probe graph, and recent failures keep a
//! "seconds ago" offset so the traffic plot can mark them on its time window.

use crate::db::ConnectionTestRecord;
use crate::support::time::{now_epoch_seconds, parse_timestamp_secs};

/// Most recent runs considered for summaries and the probe graph.
const WINDOW: usize = 30;

/// One metric's current reading plus its aggregate over the window.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MetricSummary {
    pub current: Option<f64>,
    pub mean: Option<f64>,
    pub std: Option<f64>,
    pub count: usize,
}

impl MetricSummary {
    /// `samples` are newest-first, so `current` is the most recent reading. Std
    /// is the population standard deviation (0 for a single sample).
    fn from_samples(samples: &[f64]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }
        let count = samples.len();
        let mean = samples.iter().sum::<f64>() / count as f64;
        let variance = samples
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / count as f64;
        Self {
            current: samples.first().copied(),
            mean: Some(mean),
            std: Some(variance.sqrt()),
            count,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TuiProbeHistory {
    pub icmp: MetricSummary,
    pub tcp: MetricSummary,
    pub real_delay: MetricSummary,
    pub download: MetricSummary,
    pub upload: MetricSummary,
    /// Per-latency-metric `(run_index, ms)` series, oldest→newest.
    pub icmp_points: Vec<(f64, f64)>,
    pub tcp_points: Vec<(f64, f64)>,
    pub real_delay_points: Vec<(f64, f64)>,
    /// Number of runs in the window (x-axis upper bound for the probe graph).
    pub run_count: usize,
    /// Timestamp of the most recent run, as stored.
    pub last_tested: Option<String>,
    /// Seconds-before-now (at load time) of failed runs within the window, used
    /// to place loss markers on the traffic plot's time axis.
    pub failure_secs_ago: Vec<i64>,
}

impl TuiProbeHistory {
    /// Build from recent connection tests ordered newest-first. Only the most
    /// recent [`WINDOW`] runs feed summaries and graphs; `total_runs` reflects
    /// the full history length.
    pub fn from_records(records: &[ConnectionTestRecord]) -> Self {
        let window: Vec<&ConnectionTestRecord> = records.iter().take(WINDOW).collect();
        let last_tested = records.first().map(|record| record.tested_at.clone());
        if window.is_empty() {
            return Self {
                last_tested,
                ..Self::default()
            };
        }

        let icmp = MetricSummary::from_samples(&samples(&window, |r| r.icmp_ms.map(|v| v as f64)));
        let tcp = MetricSummary::from_samples(&samples(&window, |r| r.tcp_ms.map(|v| v as f64)));
        let real_delay =
            MetricSummary::from_samples(&samples(&window, |r| r.real_delay_ms.map(|v| v as f64)));
        let download = MetricSummary::from_samples(&samples(&window, |r| r.download_mbps));
        let upload = MetricSummary::from_samples(&samples(&window, |r| r.upload_mbps));

        let run_count = window.len();
        let now = now_epoch_seconds() as i64;
        let mut icmp_points = Vec::new();
        let mut tcp_points = Vec::new();
        let mut real_delay_points = Vec::new();
        let mut failure_secs_ago = Vec::new();
        for (index, record) in window.iter().rev().enumerate() {
            let x = index as f64;
            if let Some(ms) = record.icmp_ms {
                icmp_points.push((x, ms as f64));
            }
            if let Some(ms) = record.tcp_ms {
                tcp_points.push((x, ms as f64));
            }
            if let Some(ms) = record.real_delay_ms {
                real_delay_points.push((x, ms as f64));
            }
            if record.failure_kind.is_some()
                && let Some(epoch) = parse_timestamp_secs(&record.tested_at)
            {
                let secs_ago = now - epoch;
                if secs_ago >= 0 {
                    failure_secs_ago.push(secs_ago);
                }
            }
        }

        Self {
            icmp,
            tcp,
            real_delay,
            download,
            upload,
            icmp_points,
            tcp_points,
            real_delay_points,
            run_count,
            last_tested,
            failure_secs_ago,
        }
    }
}

fn samples<F>(window: &[&ConnectionTestRecord], pick: F) -> Vec<f64>
where
    F: Fn(&ConnectionTestRecord) -> Option<f64>,
{
    window.iter().filter_map(|record| pick(record)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: i64, real_delay_ms: Option<i64>, failure: Option<&str>) -> ConnectionTestRecord {
        ConnectionTestRecord {
            id,
            run_id: None,
            config_id: 1,
            icmp_ok: None,
            icmp_ms: None,
            tcp_ok: None,
            tcp_ms: None,
            real_delay_ok: real_delay_ms.map(|_| true),
            real_delay_ms,
            download_mbps: None,
            upload_mbps: None,
            connect_ms: None,
            ttfb_ms: None,
            http_status: None,
            endpoint_ip: None,
            endpoint_location: None,
            endpoint_country: None,
            endpoint_asn: None,
            failure_kind: failure.map(str::to_string),
            failure_reason: None,
            tested_at: format!("2026-06-11T00:00:{id:02}"),
        }
    }

    #[test]
    fn summary_uses_newest_value_and_population_std() {
        // newest-first: 10, 20, 30 -> mean 20, std sqrt(200/3)
        let records = vec![
            record(3, Some(10), None),
            record(2, Some(20), None),
            record(1, Some(30), None),
        ];
        let history = TuiProbeHistory::from_records(&records);
        assert_eq!(history.real_delay.current, Some(10.0));
        assert_eq!(history.real_delay.mean, Some(20.0));
        assert_eq!(history.real_delay.count, 3);
        let std = history.real_delay.std.unwrap();
        assert!((std - (200.0f64 / 3.0).sqrt()).abs() < 1e-9);
    }

    #[test]
    fn missing_values_are_skipped_in_summary() {
        let records = vec![
            record(3, None, Some("timeout")),
            record(2, Some(40), None),
            record(1, Some(60), None),
        ];
        let history = TuiProbeHistory::from_records(&records);
        assert_eq!(history.real_delay.current, Some(40.0));
        assert_eq!(history.real_delay.count, 2);
        assert_eq!(history.real_delay.mean, Some(50.0));
    }

    #[test]
    fn empty_records_yield_default_summary() {
        let history = TuiProbeHistory::from_records(&[]);
        assert_eq!(history.real_delay, MetricSummary::default());
        assert!(history.real_delay_points.is_empty());
        assert_eq!(history.run_count, 0);
        assert!(history.last_tested.is_none());
    }

    #[test]
    fn series_are_chronological_and_totals_tracked() {
        // newest-first input; oldest=id1 -> x=0, newest=id3 -> x=2.
        let records = vec![
            record(3, Some(50), None),
            record(2, None, Some("refused")),
            record(1, Some(30), None),
        ];
        let history = TuiProbeHistory::from_records(&records);
        assert_eq!(history.run_count, 3);
        assert_eq!(history.last_tested.as_deref(), Some("2026-06-11T00:00:03"));
        assert_eq!(history.real_delay_points, vec![(0.0, 30.0), (2.0, 50.0)]);
    }
}
