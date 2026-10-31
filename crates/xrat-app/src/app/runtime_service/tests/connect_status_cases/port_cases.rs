use super::support::{import_hy2_config, import_single_config};
use super::*;
use async_trait::async_trait;
use std::ffi::OsString;
use std::io;
use std::os::unix::process::ExitStatusExt;
use std::process::{ExitStatus, Output};
use std::sync::{Arc, Mutex};
use xrat_support::process::{Child, ChildHandle, CommandSpec, ProcessSpawner, Stdio};
use xrat_support::readiness::{
    ChildPollErrorPolicy, NetworkEndpoint, PortWaiter, ReadinessError, ReadinessRequest,
    RuntimeProcessPorts, TcpConnector,
};
use xrat_support::signals::{ProcessSignal, ProcessSignals};

#[derive(Default)]
struct State {
    commands: Vec<Vec<OsString>>,
    spawned: usize,
    killed: usize,
    reaped: usize,
    running: bool,
    signals: Vec<ProcessSignal>,
    connected: Vec<NetworkEndpoint>,
    readiness: usize,
    track_tun: bool,
    tun_interface: Option<xrat_support::net::KernelInterfaceInfo>,
    delete_failures: usize,
    startup_failures: usize,
    deleted_interfaces: usize,
    version: Option<String>,
    missing_capabilities: bool,
    change_config_on_spawn: Option<std::path::PathBuf>,
}
struct FakePorts {
    state: Arc<Mutex<State>>,
    fail_readiness: bool,
    fail_validation: bool,
}
struct FakeChild(Arc<Mutex<State>>);
impl ChildHandle for FakeChild {
    fn id(&self) -> u32 {
        42
    }
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        Ok(None)
    }
    fn kill(&mut self) -> io::Result<()> {
        let mut state = self.0.lock().unwrap();
        state.killed += 1;
        state.running = false;
        Ok(())
    }
    fn wait(&mut self) -> io::Result<ExitStatus> {
        self.0.lock().unwrap().reaped += 1;
        Ok(ExitStatus::from_raw(0))
    }
}
#[async_trait]
impl ProcessSpawner for FakePorts {
    fn spawn(&self, spec: &CommandSpec) -> io::Result<Child> {
        assert!(matches!(spec.stdin, Some(Stdio::Null)));
        assert!(matches!(spec.stdout, Some(Stdio::File(_))));
        assert!(matches!(spec.stderr, Some(Stdio::File(_))));
        assert!(!spec.kill_on_drop);
        assert_eq!(spec.args[0], "run");
        assert_eq!(spec.args[1], "-c");
        assert!(std::path::Path::new(&spec.args[2]).exists());
        let mut state = self.state.lock().unwrap();
        if state.startup_failures > 0 {
            state.startup_failures -= 1;
            return Err(io::Error::other("injected startup failure"));
        }
        state.spawned += 1;
        state.running = true;
        if let Some(path) = state.change_config_on_spawn.take() {
            let contents = std::fs::read_to_string(&path).unwrap();
            std::fs::write(path, format!("{contents}\n# external edit\n")).unwrap();
        }
        let config: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&spec.args[2]).unwrap()).unwrap();
        let has_tun = config["inbounds"]
            .as_array()
            .unwrap()
            .iter()
            .any(|inbound| inbound["protocol"] == "tun" || inbound["type"] == "tun");
        if state.track_tun && has_tun {
            state.tun_interface = Some(xrat_support::net::KernelInterfaceInfo {
                name: "xrat0".into(),
                ifindex: 42,
                is_tun: true,
            });
        }
        state.commands.push(spec.args.clone());
        Ok(Child::from_handle(
            Box::new(FakeChild(self.state.clone())),
            None,
            None,
        ))
    }
    fn run(&self, spec: &CommandSpec, capture: bool) -> io::Result<Output> {
        assert!(capture);
        if spec.program == "getcap" {
            return Ok(Output {
                status: ExitStatus::from_raw(0),
                stdout: if self.state.lock().unwrap().missing_capabilities {
                    Vec::new()
                } else {
                    b"fake-xray cap_net_admin,cap_net_raw=ep\n".to_vec()
                },
                stderr: Vec::new(),
            });
        }
        self.state.lock().unwrap().commands.push(spec.args.clone());
        let version = spec.args.first().is_some_and(|arg| arg == "version");
        if !version {
            assert!(matches!(spec.stdin, Some(Stdio::Null)));
            assert!(spec.args.iter().any(|arg| arg == "-test" || arg == "check"));
        }
        Ok(Output {
            status: ExitStatus::from_raw(if !version && self.fail_validation {
                7 << 8
            } else {
                0
            }),
            stdout: if spec.args.iter().any(|arg| arg == "--name") {
                b"1.13.21".to_vec()
            } else {
                self.state
                    .lock()
                    .unwrap()
                    .version
                    .clone()
                    .unwrap_or_else(|| "Xray 26.7.28".into())
                    .into_bytes()
            },
            stderr: if self.fail_validation {
                b"invalid fixture".to_vec()
            } else {
                Vec::new()
            },
        })
    }
    async fn output_async(&self, spec: &CommandSpec) -> io::Result<Output> {
        self.run(spec, true)
    }
}
impl xrat_support::net::TunInterfaceOps for FakePorts {
    fn policy_rules(&self) -> io::Result<Vec<xrat_support::net::KernelPolicyRule>> {
        if self.state.lock().unwrap().tun_interface.is_none() {
            return Ok(vec![]);
        }
        Ok([2, 10]
            .into_iter()
            .map(|family| xrat_support::net::KernelPolicyRule {
                family,
                destination_prefix: 0,
                source_prefix: 0,
                tos: 0,
                table: 0,
                action: 3,
                flags: 0,
                attributes: vec![xrat_support::net::KernelRuleAttribute {
                    kind: 6,
                    value: 9010u32.to_ne_bytes().to_vec(),
                }],
            })
            .collect())
    }
    fn delete_policy_rule(&self, _rule: &xrat_support::net::KernelPolicyRule) -> io::Result<()> {
        Ok(())
    }
    fn inspect_interface(
        &self,
        name: &str,
    ) -> io::Result<Option<xrat_support::net::KernelInterfaceInfo>> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .tun_interface
            .clone()
            .filter(|info| info.name == name))
    }

    fn delete_interface(&self, _name: &str, expected_ifindex: u32) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        if state.delete_failures > 0 {
            state.delete_failures -= 1;
            return Err(io::Error::other("injected cleanup failure"));
        }
        assert_eq!(
            state.tun_interface.as_ref().unwrap().ifindex,
            expected_ifindex
        );
        state.tun_interface = None;
        state.deleted_interfaces += 1;
        Ok(())
    }
}
#[async_trait]
impl PortWaiter for FakePorts {
    async fn wait(
        &self,
        child: &mut dyn ChildHandle,
        request: ReadinessRequest,
    ) -> Result<(), ReadinessError> {
        assert_eq!(child.id(), 42);
        assert!(matches!(request.child_errors, ChildPollErrorPolicy::Fail));
        assert_eq!(request.endpoints.len(), 1);
        self.state.lock().unwrap().readiness += 1;
        if self.fail_readiness {
            Err(ReadinessError::Timeout {
                port: request.endpoints[0].port,
            })
        } else {
            Ok(())
        }
    }
}
impl ProcessSignals for FakePorts {
    fn send(&self, pid: i64, signal: ProcessSignal) -> io::Result<bool> {
        assert_eq!(pid, 42);
        let mut state = self.state.lock().unwrap();
        state.signals.push(signal);
        let running = state.running;
        if signal == ProcessSignal::Term || signal == ProcessSignal::Kill {
            state.running = false;
        }
        Ok(running)
    }
}
#[async_trait]
impl TcpConnector for FakePorts {
    async fn connect(&self, endpoint: &NetworkEndpoint) -> io::Result<()> {
        self.state.lock().unwrap().connected.push(endpoint.clone());
        Ok(())
    }
}
struct FakeResolver;
impl xrat_support::net::HostResolver for FakeResolver {
    fn resolve(&self, _host: &str, _port: u16) -> Option<std::net::IpAddr> {
        Some(std::net::IpAddr::V4(std::net::Ipv4Addr::new(
            198, 51, 100, 1,
        )))
    }
}

fn ports(fail_readiness: bool, fail_validation: bool) -> (RuntimeProcessPorts, Arc<Mutex<State>>) {
    let state = Arc::new(Mutex::new(State::default()));
    let fake = Arc::new(FakePorts {
        state: state.clone(),
        fail_readiness,
        fail_validation,
    });
    (
        RuntimeProcessPorts {
            spawner: fake.clone(),
            waiter: fake.clone(),
            signals: fake.clone(),
            connector: fake.clone(),
            tun: fake,
            resolver: Arc::new(FakeResolver),
            firewall: Arc::new(xrat_support::net::MockFirewallOps::default()),
        },
        state,
    )
}

#[tokio::test]
async fn injected_runtime_ports_cover_preflight_start_status_and_stop_for_both_engines() {
    for singbox in [false, true] {
        let mut context = test_context().await;
        let config = if singbox {
            context.app_config.runtime.engine = "sing-box".into();
            import_hy2_config(&context).await
        } else {
            import_single_config(&context).await
        };
        let (ports, state) = ports(false, false);
        let service = RuntimeService::with_process_ports(&context, ports);
        let connected = service
            .connect(ConnectRequest {
                config_id: config.id,
            })
            .await
            .unwrap();
        assert_eq!(connected.pid, 42);
        let snapshot = service.status().await.unwrap();
        assert!(snapshot.pid_running);
        assert!(!snapshot.inbound_health.has_unreachable_endpoint());
        assert!(service.disconnect().await.unwrap().stopped_session);
        let latest = context
            .db
            .get_latest_runtime_session()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(latest.status, RuntimeSessionStatus::Stopped);
        assert!(context.db.get_active_config().await.unwrap().is_none());
        let state = state.lock().unwrap();
        assert_eq!(state.spawned, 1);
        assert_eq!(state.readiness, 1);
        assert!(!state.connected.is_empty());
        assert!(state.signals.contains(&ProcessSignal::Term));
        assert_eq!(
            state.killed, 0,
            "managed success must detach child ownership"
        );
        assert_eq!(state.reaped, 0);
    }
}

#[tokio::test]
async fn failed_injected_startup_reaps_child_and_persists_failed_session() {
    for singbox in [false, true] {
        let mut context = test_context().await;
        let config = if singbox {
            context.app_config.runtime.engine = "sing-box".into();
            import_hy2_config(&context).await
        } else {
            import_single_config(&context).await
        };
        let (ports, state) = ports(true, false);
        assert!(
            RuntimeService::with_process_ports(&context, ports)
                .connect(ConnectRequest {
                    config_id: config.id
                })
                .await
                .is_err()
        );
        let latest = context
            .db
            .get_latest_runtime_session()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(latest.status, RuntimeSessionStatus::Failed);
        assert!(context.db.get_active_config().await.unwrap().is_none());
        let state = state.lock().unwrap();
        assert_eq!((state.spawned, state.killed, state.reaped), (1, 1, 1));
    }
}

#[tokio::test]
async fn injected_preflight_failure_does_not_spawn_or_create_session() {
    let context = test_context().await;
    let config = import_single_config(&context).await;
    let (ports, state) = ports(false, true);
    let error = RuntimeService::with_process_ports(&context, ports)
        .connect(ConnectRequest {
            config_id: config.id,
        })
        .await
        .err()
        .unwrap();
    assert!(error.to_string().contains("invalid fixture"));
    assert_eq!(state.lock().unwrap().spawned, 0);
    assert!(
        context
            .db
            .get_latest_runtime_session()
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn tun_handoffs_preserve_ownership_and_restore_after_cleanup_or_startup_failure() {
    for rotation in [false, true] {
        for failure in [None, Some("cleanup"), Some("startup")] {
            let mut context = test_context().await;
            context.app_config.runtime.tun.enabled = true;
            context.app_config.runtime.replace_active_session = true;
            let original = import_single_config(&context).await;
            import_hy2_config(&context).await;
            let next = context
                .db
                .list_configs(&Default::default())
                .await
                .unwrap()
                .into_iter()
                .find(|config| config.id != original.id)
                .unwrap();
            let (ports, state) = ports(false, false);
            state.lock().unwrap().track_tun = true;
            let service = RuntimeService::with_process_ports(&context, ports);
            service
                .connect(ConnectRequest {
                    config_id: original.id,
                })
                .await
                .unwrap();
            {
                let mut state = state.lock().unwrap();
                state.delete_failures = usize::from(failure == Some("cleanup"));
                state.startup_failures = usize::from(failure == Some("startup"));
            }
            let result = if rotation {
                service
                    .replace(ReplaceRequest {
                        trigger: crate::app::services::rotation::RotationTrigger::Manual,
                        candidate_id: Some(next.id),
                    })
                    .await
                    .map(|result| result.new_session_id)
            } else {
                service
                    .connect(ConnectRequest { config_id: next.id })
                    .await
                    .map(|result| result.session_id)
            };
            if failure.is_some() {
                let error = result.unwrap_err().to_string();
                assert!(error.contains("previous runtime was restored"), "{error}");
            } else {
                result.unwrap();
            }
            let active = context.db.get_active_config().await.unwrap().unwrap();
            assert_eq!(
                active.id,
                if failure.is_some() {
                    original.id
                } else {
                    next.id
                }
            );
            assert!(state.lock().unwrap().running);
            assert!(state.lock().unwrap().deleted_interfaces > 0);
            let record = crate::app::runtime_service::tun_ownership::load_ownership(
                &context.runtime_paths.runtime_dir,
            )
            .unwrap();
            assert_eq!(record.ifindex, Some(42));
            service.disconnect().await.unwrap();
        }
    }
}

#[tokio::test]
async fn tun_handoffs_refuse_changed_interface_identity_before_stopping_runtime() {
    for rotation in [false, true] {
        let mut context = test_context().await;
        context.app_config.runtime.tun.enabled = true;
        context.app_config.runtime.replace_active_session = true;
        let original = import_single_config(&context).await;
        import_hy2_config(&context).await;
        let next = context
            .db
            .list_configs(&Default::default())
            .await
            .unwrap()
            .into_iter()
            .find(|config| config.id != original.id)
            .unwrap();
        let (ports, state) = ports(false, false);
        state.lock().unwrap().track_tun = true;
        let service = RuntimeService::with_process_ports(&context, ports);
        service
            .connect(ConnectRequest {
                config_id: original.id,
            })
            .await
            .unwrap();
        state
            .lock()
            .unwrap()
            .tun_interface
            .as_mut()
            .unwrap()
            .ifindex = 99;
        let result = if rotation {
            service
                .replace(ReplaceRequest {
                    trigger: crate::app::services::rotation::RotationTrigger::Manual,
                    candidate_id: Some(next.id),
                })
                .await
                .map(|result| result.new_session_id)
        } else {
            service
                .connect(ConnectRequest { config_id: next.id })
                .await
                .map(|result| result.session_id)
        };
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("does not match previously recorded interface index")
        );
        assert!(state.lock().unwrap().running);
        assert!(
            !state
                .lock()
                .unwrap()
                .signals
                .iter()
                .any(|signal| matches!(signal, ProcessSignal::Term | ProcessSignal::Kill))
        );
        assert_eq!(
            context.db.get_active_config().await.unwrap().unwrap().id,
            original.id
        );
        assert!(
            service
                .disconnect()
                .await
                .unwrap_err()
                .to_string()
                .contains("does not match previously recorded interface index")
        );
        assert_eq!(state.lock().unwrap().deleted_interfaces, 0);
        assert!(
            crate::app::runtime_service::tun_ownership::load_ownership(
                &context.runtime_paths.runtime_dir
            )
            .is_some()
        );
        state
            .lock()
            .unwrap()
            .tun_interface
            .as_mut()
            .unwrap()
            .ifindex = 42;
        service.disconnect().await.unwrap();
    }
}

#[path = "tun_mode_cases.rs"]
mod tun_mode_cases;
