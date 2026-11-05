use crate::app::read_models::TrafficSnapshot;
use crate::app::{
    context::AppContext,
    runtime_service::{RuntimeService, RuntimeStatusSnapshot},
};
use std::time::Duration;
use xrat_engines::xray::stats::{SingboxStatsSource, StatsSource, XrayStatsSource};
use xrat_web::types::Traffic;

pub(crate) async fn collect(context: &AppContext) -> crate::app::Result<TrafficSnapshot> {
    #[cfg(any(windows, test))]
    if let Some(client) = &context.machine_client {
        return client.traffic().await;
    }
    let mut current = context.clone();
    current.app_config = crate::app::config::load(&context.runtime_paths.config_path)?;
    if !current.app_config.runtime.stats.enabled {
        return Ok(TrafficSnapshot {
            session_id: None,
            traffic: None,
        });
    }
    let snapshot = RuntimeService::new(&current).status().await?;
    let session = snapshot
        .session
        .as_ref()
        .filter(|_| snapshot.pid_running)
        .map(|session| session.id);
    let traffic = sample(&current, &snapshot).await;
    let latest = current
        .db
        .get_latest_runtime_session()
        .await?
        .map(|session| session.id);
    if latest != session {
        return Ok(TrafficSnapshot {
            session_id: None,
            traffic: None,
        });
    }
    Ok(TrafficSnapshot {
        session_id: session,
        traffic,
    })
}

pub(crate) async fn sample(
    context: &AppContext,
    snapshot: &RuntimeStatusSnapshot,
) -> Option<Traffic> {
    #[cfg(any(windows, test))]
    if let Some(client) = &context.machine_client {
        return client
            .traffic()
            .await
            .ok()
            .filter(|remote| {
                remote.session_id
                    == snapshot
                        .session
                        .as_ref()
                        .filter(|_| snapshot.pid_running)
                        .map(|session| session.id)
            })
            .and_then(|remote| remote.traffic);
    }
    if !snapshot.pid_running
        || !context.app_config.runtime.stats.enabled
        || snapshot
            .session_config
            .as_ref()
            .is_some_and(xrat_db::ConfigRecord::is_xray_profile)
    {
        return None;
    }
    let settings = &context.app_config.runtime.stats;
    let source: Box<dyn StatsSource> = if context.app_config.runtime.engine == "sing-box"
        || snapshot
            .active_config
            .as_ref()
            .is_some_and(|config| config.protocol == "hysteria2")
    {
        Box::new(SingboxStatsSource::new(
            &format!("{}:{}", settings.host, settings.port),
            None,
        ))
    } else {
        Box::new(XrayStatsSource::new(&settings.host, settings.port))
    };
    tokio::time::timeout(Duration::from_millis(500), source.sample())
        .await
        .ok()
        .and_then(Result::ok)
        .map(|sample| Traffic {
            uploaded_bytes: sample.uplink_total,
            downloaded_bytes: sample.downlink_total,
        })
}
