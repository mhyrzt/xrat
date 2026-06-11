//! Bounded traffic-stats history for the TUI stats tab. Successive
//! [`StatsSample`]s are differenced into per-tick throughput and kept in a ring
//! buffer for the sparkline. The buffer resets whenever the runtime session id
//! changes so totals never carry across sessions.

use std::collections::VecDeque;

use crate::xray::stats::StatsSample;

/// Upper bound on retained points (~2 minutes at a 1s poll interval).
const CAPACITY: usize = 120;

/// One retained reading: cumulative totals plus the throughput since the
/// previous sample, in bytes per tick (~bytes/sec at the 1s poll interval).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StatsPoint {
    pub uplink_total: u64,
    pub downlink_total: u64,
    pub up_rate: u64,
    pub down_rate: u64,
}

#[derive(Debug, Default)]
pub struct StatsHistory {
    points: VecDeque<StatsPoint>,
    session_id: Option<i64>,
    last: Option<StatsSample>,
}

impl StatsHistory {
    /// Record a sample for `session_id`. A changed (or first) session id clears
    /// prior history so totals and the graph restart cleanly.
    pub fn record(&mut self, session_id: Option<i64>, sample: StatsSample) {
        if session_id != self.session_id {
            self.clear();
            self.session_id = session_id;
        }

        let up_rate = self
            .last
            .map(|last| sample.uplink_total.saturating_sub(last.uplink_total))
            .unwrap_or(0);
        let down_rate = self
            .last
            .map(|last| sample.downlink_total.saturating_sub(last.downlink_total))
            .unwrap_or(0);
        self.last = Some(sample);

        self.points.push_back(StatsPoint {
            uplink_total: sample.uplink_total,
            downlink_total: sample.downlink_total,
            up_rate,
            down_rate,
        });
        while self.points.len() > CAPACITY {
            self.points.pop_front();
        }
    }

    /// Drop all retained points and counters (view-only clear).
    pub fn clear(&mut self) {
        self.points.clear();
        self.last = None;
        self.session_id = None;
    }

    pub fn latest(&self) -> Option<&StatsPoint> {
        self.points.back()
    }

    pub fn down_rates(&self) -> Vec<u64> {
        self.points.iter().map(|point| point.down_rate).collect()
    }

    pub fn up_rates(&self) -> Vec<u64> {
        self.points.iter().map(|point| point.up_rate).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(up: u64, down: u64) -> StatsSample {
        StatsSample {
            uplink_total: up,
            downlink_total: down,
        }
    }

    #[test]
    fn differences_successive_samples_into_rates() {
        let mut history = StatsHistory::default();
        history.record(Some(1), sample(100, 1000));
        history.record(Some(1), sample(150, 1900));
        let latest = history.latest().unwrap();
        assert_eq!(latest.up_rate, 50);
        assert_eq!(latest.down_rate, 900);
        assert_eq!(latest.uplink_total, 150);
    }

    #[test]
    fn resets_history_when_session_changes() {
        let mut history = StatsHistory::default();
        history.record(Some(1), sample(100, 1000));
        history.record(Some(2), sample(5, 5));
        assert_eq!(history.down_rates().len(), 1);
        let latest = history.latest().unwrap();
        assert_eq!(latest.up_rate, 0);
        assert_eq!(latest.downlink_total, 5);
    }

    #[test]
    fn rolls_over_at_capacity() {
        let mut history = StatsHistory::default();
        for index in 0..(CAPACITY as u64 + 50) {
            history.record(Some(1), sample(index, index * 2));
        }
        assert_eq!(history.down_rates().len(), CAPACITY);
    }
}
