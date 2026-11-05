use std::time::Duration;

use crate::app::context::AppContext;
use crate::app::daemon::supervisor::SupervisorState;
use crate::app::runtime_service::RuntimeService;
use xrat_prober::AcceptedHttpStatuses;
use xrat_support::time::now_epoch_seconds;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct HealthTickOutcome {
    pub health_failure_recorded: bool,
    pub timer_due: bool,
    pub cooldown_active: bool,
    pub probe: Option<HealthProbe>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HealthProbe {
    pub session_id: i64,
    proxy_url: String,
    test_url: String,
    timeout_ms: u64,
    accepted_statuses: AcceptedHttpStatuses,
    follow_redirects: bool,
}

pub(super) async fn handle_health_tick(
    state: &mut SupervisorState,
    context: &AppContext,
) -> HealthTickOutcome {
    let now = now_epoch_seconds();
    let snapshot = match RuntimeService::new(context).status().await {
        Ok(snapshot) => snapshot,
        Err(error) => {
            tracing::debug!(operation = "health_runtime_status", %error, "failed to load runtime health status");
            return HealthTickOutcome::default();
        }
    };
    let Some(session) = snapshot.session else {
        return HealthTickOutcome::default();
    };
    if snapshot
        .session_config
        .as_ref()
        .is_some_and(xrat_db::ConfigRecord::is_xray_profile)
    {
        return HealthTickOutcome::default();
    }
    let timer_due = state.rotation_enabled
        && snapshot.pid_running
        && state
            .next_timer_epoch_secs
            .is_some_and(|next_timer| now >= next_timer);

    if !snapshot.pid_running || snapshot.inbound_health.has_unreachable_endpoint() {
        let reason = if snapshot.pid_running {
            "runtime_inbound_unreachable"
        } else {
            "runtime_process_exited"
        };
        if !should_record_health_failure(&session) {
            return HealthTickOutcome {
                timer_due,
                cooldown_active: true,
                ..Default::default()
            };
        }
        crate::app::services::runtime_transitions::RuntimeTransitionService::new(
            context,
            &state.instance_id,
        )
        .record_health_failure(&session, state.cooldown_secs, reason)
        .await;
        return HealthTickOutcome {
            health_failure_recorded: true,
            timer_due,
            ..Default::default()
        };
    }

    if !state.health_trigger_enabled || state.health_probe_in_flight {
        return HealthTickOutcome {
            timer_due,
            ..Default::default()
        };
    }

    let proxy_url = if let (Some(host), Some(port)) = (&session.socks_host, session.socks_port) {
        Some(format!("socks5h://{}:{port}", probe_host(host)))
    } else if let (Some(host), Some(port)) = (&session.http_host, session.http_port) {
        Some(format!("http://{}:{port}", probe_host(host)))
    } else {
        None
    };
    let Some(proxy_url) = proxy_url else {
        if session.shadowsocks_port.is_some() && state.last_health_error.is_none() {
            state.last_health_error = Some(
                "Shadowsocks-only runtime health uses process and inbound-socket checks"
                    .to_string(),
            );
        }
        return HealthTickOutcome {
            timer_due,
            ..Default::default()
        };
    };

    let settings = &context.app_config.testing.real_delay;
    let accepted_statuses = match (
        &settings.accepted_status_codes,
        &settings.accepted_status_ranges,
    ) {
        (None, None) => AcceptedHttpStatuses::default(),
        (codes, ranges) => AcceptedHttpStatuses::new(
            codes.clone().unwrap_or_default(),
            ranges
                .as_ref()
                .map(|ranges| ranges.iter().map(|range| range.bounds()).collect())
                .unwrap_or_default(),
        )
        .unwrap_or_else(|error| {
            tracing::debug!(operation = "health_accepted_statuses", session_id = session.id, %error, "using default health status matcher");
            AcceptedHttpStatuses::default()
        }),
    };
    state.health_probe_in_flight = true;
    HealthTickOutcome {
        timer_due,
        probe: Some(HealthProbe {
            session_id: session.id,
            proxy_url,
            test_url: settings.url.clone(),
            timeout_ms: settings.timeout,
            accepted_statuses,
            follow_redirects: settings.follow_redirects,
        }),
        ..Default::default()
    }
}

pub(super) async fn execute_probe(probe: HealthProbe) -> (i64, bool, Option<String>) {
    let result = xrat_prober::real_delay::make_proxied_request_via(
        &probe.proxy_url,
        &probe.test_url,
        Duration::from_millis(probe.timeout_ms),
        &probe.accepted_statuses,
        probe.follow_redirects,
    )
    .await;
    (probe.session_id, result.success, result.failure_reason)
}

pub(super) async fn handle_probe_completed(
    state: &mut SupervisorState,
    context: &AppContext,
    session_id: i64,
    success: bool,
    error: Option<String>,
) -> bool {
    state.health_probe_in_flight = false;
    state.last_health_check_epoch_secs = Some(now_epoch_seconds());
    let current = context
        .db
        .get_running_runtime_session()
        .await
        .unwrap_or_else(|error| {
            tracing::debug!(operation = "health_probe_session", %error, "failed to load optional supervisor state");
            None
        });
    if current.as_ref().map(|session| session.id) != Some(session_id) {
        return false;
    }
    if success {
        state.consecutive_health_failures = 0;
        state.last_health_error = None;
        state.pending_health_recovery = false;
        return false;
    }

    state.consecutive_health_failures = state.consecutive_health_failures.saturating_add(1);
    state.last_health_error = error;
    if state.consecutive_health_failures < state.health_failure_threshold {
        return false;
    }
    let Some(session) = current else {
        return false;
    };
    if !should_record_health_failure(&session) {
        return false;
    }
    state.pending_health_recovery = true;
    state.consecutive_health_failures = 0;
    crate::app::services::runtime_transitions::RuntimeTransitionService::new(
        context,
        &state.instance_id,
    )
    .record_health_failure(&session, state.cooldown_secs, "runtime_data_plane_failed")
    .await;
    true
}

fn probe_host(host: &str) -> String {
    match host {
        "0.0.0.0" => "127.0.0.1".to_string(),
        "::" | "[::]" => "[::1]".to_string(),
        value if value.starts_with('[') && value.ends_with(']') => value.to_string(),
        value if value.contains(':') => format!("[{value}]"),
        value => value.to_string(),
    }
}

pub(super) fn should_record_health_failure(session: &xrat_db::RuntimeSessionRecord) -> bool {
    let Some(cooldown_until) = session.cooldown_until.as_deref() else {
        return true;
    };
    let Ok(cooldown_until) = cooldown_until.parse::<u64>() else {
        return true;
    };
    now_epoch_seconds() >= cooldown_until
}
