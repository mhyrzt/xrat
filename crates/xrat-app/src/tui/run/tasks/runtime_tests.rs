use super::*;
use crate::app::daemon::ipc::{DaemonResponse, DaemonResponseCode, TunStatePayload};
use crate::app::read_models::RuntimeOverview;
use crate::app::runtime_service::{
    RuntimeEndpoint, RuntimeEndpointHealth, RuntimeEndpointState, RuntimeService,
};
use crate::app::services::dashboard::DashboardService;
use crate::app::services::machine_service::{
    GATEWAY_VERSION, GatewayPayload, GatewayRequest, MachineClient, Transport,
};
use crate::app::services::test_support::FakeClock;
use crate::app::tests::TestAppBuilder;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn failed_runtime_poll_reports_completion_and_next_poll_can_succeed() {
    let (context, _root) = TestAppBuilder::new("runtime-poll-retry")
        .build_with_root()
        .await;
    let (sender, mut receiver) = mpsc::unbounded_channel();
    spawn_poll_runtime(context.clone(), &sender);
    assert!(
        tokio::time::timeout(Duration::from_secs(2), receiver.recv())
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    std::fs::write(&context.runtime_paths.config_path, "").unwrap();
    spawn_poll_runtime(context, &sender);
    let status = tokio::time::timeout(Duration::from_secs(2), receiver.recv())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(!status.pid_running);
}

struct OverviewTransport(RuntimeOverview);

#[async_trait::async_trait]
impl Transport for OverviewTransport {
    async fn exchange(
        &self,
        _: GatewayRequest,
    ) -> crate::app::Result<DaemonResponse<GatewayPayload>> {
        Ok(DaemonResponse {
            protocol_version: GATEWAY_VERSION,
            ok: true,
            code: DaemonResponseCode::Ok,
            message: "owner overview".into(),
            payload: Some(GatewayPayload::Overview(Box::new(self.0.clone()))),
        })
    }
}

struct FixedLocalIp;

impl xrat_support::net::LocalIpResolver for FixedLocalIp {
    fn primary_ip(&self) -> Option<std::net::IpAddr> {
        Some("192.0.2.10".parse().unwrap())
    }
}

#[tokio::test]
async fn runtime_poll_preserves_machine_tun_state_and_shared_endpoint_projection() {
    let (mut context, _root) = TestAppBuilder::new("runtime-poll-machine")
        .build_with_root()
        .await;
    let mut runtime = RuntimeService::new(&context).status().await.unwrap();
    runtime.inbound_health.http = Some(RuntimeEndpointHealth {
        endpoint: RuntimeEndpoint {
            host: "0.0.0.0".into(),
            port: 8080,
        },
        state: RuntimeEndpointState::Reachable,
    });
    context.machine_client = Some(MachineClient::with_transport(Arc::new(OverviewTransport(
        RuntimeOverview {
            runtime,
            tun: TunStatePayload {
                enabled: true,
                active: true,
                engine: "sing-box".into(),
                interface: Some("owner-tun".into()),
                active_config_ref: None,
                session_id: None,
                split_mode: "whitelist".into(),
                blacklist_count: 0,
                whitelist_count: 2,
                kill_switch: "off".into(),
                kill_switch_active: false,
                allow_lan: true,
            },
            daemon_ready: true,
            rotation_enabled: false,
            rotation_interval_secs: 60,
            test_stages: vec!["tcp".into()],
            config_path: "owner-config.toml".into(),
        },
    ))));
    let service =
        DashboardService::with_dependencies(&context, FakeClock::at(1000), Arc::new(FixedLocalIp));
    let projected: TuiRuntimeStatus = service.load_runtime().await.unwrap().into();
    assert_eq!(projected.http.as_deref(), Some("http://192.0.2.10:8080 ✓"));
    assert_eq!(
        projected.tun.as_deref(),
        Some("owner-tun (sing-box) · split: whitelist (2)")
    );
    assert!(projected.tun_active);

    let (sender, mut receiver) = mpsc::unbounded_channel();
    spawn_poll_runtime(context, &sender);
    let polled = tokio::time::timeout(Duration::from_secs(2), receiver.recv())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(polled.tun, projected.tun);
    assert!(polled.tun_active);
}
