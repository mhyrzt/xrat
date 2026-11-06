use crate::app::{context::AppContext, runtime_service::RuntimeStatusSnapshot};
use xrat_web::types::{DashboardData, LatencyMetric, ProbePoint};

pub(crate) async fn collect(context: &AppContext) -> crate::app::Result<DashboardData> {
    #[cfg(any(windows, test))]
    if let Some(client) = &context.machine_client {
        return client.dashboard().await;
    }
    let mut current = context.clone();
    current.app_config = crate::app::config::load(&context.runtime_paths.config_path)?;
    let snapshot = crate::app::runtime_service::RuntimeService::new(&current)
        .status()
        .await?;
    load(&current, &snapshot).await
}

pub(crate) async fn load(
    context: &AppContext,
    snapshot: &RuntimeStatusSnapshot,
) -> crate::app::Result<DashboardData> {
    #[cfg(any(windows, test))]
    if let Some(client) = &context.machine_client {
        return client.dashboard().await;
    }
    let db = &context.db;
    let sampled_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1000.0;
    let testing = &context.app_config.testing;
    let enabled_metrics = [
        (LatencyMetric::Tcp, testing.tcp.enabled),
        (LatencyMetric::Icmp, testing.icmp.enabled),
        (LatencyMetric::RealDelay, testing.real_delay.enabled),
    ]
    .into_iter()
    .filter_map(|(metric, enabled)| enabled.then_some(metric))
    .collect();
    let probes = if let Some(config) = snapshot
        .active_config
        .as_ref()
        .filter(|_| snapshot.pid_running)
    {
        db.recent_connection_tests(config.id, 30)
            .await?
            .into_iter()
            .rev()
            .map(|record| ProbePoint {
                id: record.id,
                tested_at: record.tested_at,
                icmp_ms: successful(record.icmp_ok, record.icmp_ms),
                tcp_ms: successful(record.tcp_ok, record.tcp_ms),
                real_delay_ms: successful(record.real_delay_ok, record.real_delay_ms),
            })
            .collect()
    } else {
        Vec::new()
    };
    let tun = crate::app::services::tun::capture_state(
        context,
        snapshot,
        &xrat_support::readiness::RuntimeProcessPorts::default(),
    );
    let supported = cfg!(any(target_os = "linux", target_os = "macos"))
        && matches!(
            context.app_config.runtime.engine.as_str(),
            "xray" | "sing-box"
        )
        && (!cfg!(target_os = "macos") || context.app_config.runtime.engine == "sing-box");
    Ok(DashboardData {
        sampled_at_ms,
        session_id: snapshot
            .session
            .as_ref()
            .filter(|_| snapshot.pid_running)
            .map(|session| session.id),
        runtime_status: snapshot.status.as_str().into(),
        stats_enabled: context.app_config.runtime.stats.enabled
            && !snapshot
                .session_config
                .as_ref()
                .is_some_and(xrat_db::ConfigRecord::is_xray_profile),
        tun_enabled: tun.enabled,
        tun_active: tun.active,
        tun_available: supported,
        tun_reason: (!supported).then(|| {
            "Managed TUN requires Linux with Xray/sing-box, or macOS with sing-box.".into()
        }),
        enabled_metrics,
        last_subscription_refresh: db
            .last_subscription_refresh((sampled_at_ms / 1000.0) as i64)
            .await?,
        probes,
        traffic_history: Vec::new(),
    })
}

fn successful(ok: Option<bool>, value: Option<i64>) -> Option<i64> {
    value.filter(|value| *value >= 0 && ok == Some(true))
}
