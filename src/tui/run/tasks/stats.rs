use tokio::sync::mpsc;

use crate::xray::stats::{SingboxStatsSource, StatsSample, StatsSource, XrayStatsSource};

/// Which stats backend to sample for the active session. Resolved from the
/// active config protocol and the configured runtime engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatsEngine {
    Xray,
    Singbox,
}

/// Sample the engine stats endpoint once and forward the reading tagged with the
/// session id. Sampling is best-effort: transport or decode errors are dropped
/// so a missing endpoint never disturbs the UI.
pub fn spawn_poll_stats(
    engine: StatsEngine,
    host: String,
    port: u16,
    session_id: Option<i64>,
    stats_tx: &mpsc::UnboundedSender<(Option<i64>, StatsSample)>,
) {
    let stats_tx = stats_tx.clone();
    tokio::spawn(async move {
        let source: Box<dyn StatsSource> = match engine {
            StatsEngine::Xray => Box::new(XrayStatsSource::new(&host, port)),
            StatsEngine::Singbox => {
                Box::new(SingboxStatsSource::new(&format!("{host}:{port}"), None))
            }
        };
        if let Ok(sample) = source.sample().await {
            let _ = stats_tx.send((session_id, sample));
        }
    });
}
