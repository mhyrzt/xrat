//! Engine-neutral traffic stats sampling for the TUI stats tab. A
//! [`StatsSource`] yields cumulative uplink/downlink byte counters that the
//! poller turns into totals and throughput. Sampling is best-effort: callers
//! treat any [`StatsError`] as "no sample this tick" and keep the previous view.

mod singbox;
mod xray;

pub use singbox::SingboxStatsSource;
pub use xray::XrayStatsSource;

/// A single cumulative traffic reading. `uplink_total` and `downlink_total` are
/// monotonically increasing byte counters for the lifetime of the engine
/// session; throughput is derived by differencing successive samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatsSample {
    pub uplink_total: u64,
    pub downlink_total: u64,
}

#[derive(Debug, thiserror::Error)]
#[error("stats sampling failed: {0}")]
pub struct StatsError(pub String);

#[async_trait::async_trait]
pub trait StatsSource: Send + Sync {
    async fn sample(&self) -> Result<StatsSample, StatsError>;
}
