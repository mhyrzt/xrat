use crate::process::ChildHandle;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkEndpoint {
    pub host: String,
    pub port: u16,
}
#[allow(
    clippy::double_must_use,
    reason = "async_trait adds must_use to methods returning already must-use boxed futures"
)]
#[async_trait]
pub trait TcpConnector: Send + Sync {
    async fn connect(&self, endpoint: &NetworkEndpoint) -> std::io::Result<()>;
}
pub struct TokioTcpConnector;
#[async_trait]
impl TcpConnector for TokioTcpConnector {
    async fn connect(&self, endpoint: &NetworkEndpoint) -> std::io::Result<()> {
        tokio::net::TcpStream::connect((endpoint.host.as_str(), endpoint.port))
            .await
            .map(|_| ())
    }
}
#[derive(Debug, Clone, Copy)]
pub enum ChildPollErrorPolicy {
    Fail,
    Ignore,
}
pub struct ReadinessRequest {
    pub endpoints: Vec<NetworkEndpoint>,
    pub timeout: Duration,
    pub poll_interval: Duration,
    pub child_errors: ChildPollErrorPolicy,
}
impl ReadinessRequest {
    pub fn single(host: &str, port: u16, timeout: Duration) -> Self {
        Self {
            endpoints: vec![NetworkEndpoint {
                host: host.into(),
                port,
            }],
            timeout,
            poll_interval: Duration::from_millis(100),
            child_errors: ChildPollErrorPolicy::Fail,
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum ReadinessError {
    #[error("failed to inspect process: {0}")]
    Io(#[from] std::io::Error),
    #[error("process exited during startup: {0}")]
    ProcessExited(std::process::ExitStatus),
    #[error("port {port} was not ready before startup timeout")]
    Timeout { port: u16 },
}
#[allow(
    clippy::double_must_use,
    reason = "async_trait adds must_use to methods returning already must-use boxed futures"
)]
#[async_trait]
pub trait PortWaiter: Send + Sync {
    async fn wait(
        &self,
        child: &mut dyn ChildHandle,
        request: ReadinessRequest,
    ) -> Result<(), ReadinessError>;
}
pub struct TcpPortWaiter {
    connector: Arc<dyn TcpConnector>,
}
impl Default for TcpPortWaiter {
    fn default() -> Self {
        Self::new(Arc::new(TokioTcpConnector))
    }
}
impl TcpPortWaiter {
    pub fn new(connector: Arc<dyn TcpConnector>) -> Self {
        Self { connector }
    }
}
#[async_trait]
impl PortWaiter for TcpPortWaiter {
    async fn wait(
        &self,
        child: &mut dyn ChildHandle,
        request: ReadinessRequest,
    ) -> Result<(), ReadinessError> {
        let start = tokio::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => return Err(ReadinessError::ProcessExited(status)),
                Err(error) if matches!(request.child_errors, ChildPollErrorPolicy::Fail) => {
                    return Err(error.into());
                }
                _ => {}
            }
            let mut unavailable = None;
            for endpoint in &request.endpoints {
                let remaining = request.timeout.saturating_sub(start.elapsed());
                let attempt_timeout = remaining.min(request.poll_interval);
                if !matches!(
                    tokio::time::timeout(attempt_timeout, self.connector.connect(endpoint)).await,
                    Ok(Ok(()))
                ) {
                    unavailable = Some(endpoint.port);
                    break;
                }
            }
            let Some(port) = unavailable else {
                return Ok(());
            };
            if start.elapsed() >= request.timeout {
                return Err(ReadinessError::Timeout { port });
            }
            tokio::time::sleep(
                request
                    .poll_interval
                    .min(request.timeout.saturating_sub(start.elapsed())),
            )
            .await;
        }
    }
}

#[derive(Clone)]
pub struct RuntimeProcessPorts {
    pub spawner: Arc<dyn crate::process::ProcessSpawner>,
    pub waiter: Arc<dyn PortWaiter>,
    pub signals: Arc<dyn crate::signals::ProcessSignals>,
    pub connector: Arc<dyn TcpConnector>,
    pub tun: Arc<dyn crate::net::TunInterfaceOps>,
    pub resolver: Arc<dyn crate::net::HostResolver>,
    pub firewall: Arc<dyn crate::net::FirewallOps>,
}
impl Default for RuntimeProcessPorts {
    fn default() -> Self {
        Self {
            spawner: Arc::new(crate::process::SystemProcessSpawner),
            waiter: Arc::new(TcpPortWaiter::default()),
            signals: Arc::new(crate::signals::SystemProcessSignals::default()),
            connector: Arc::new(TokioTcpConnector),
            tun: Arc::new(crate::net::SystemTunInterfaceOps),
            resolver: Arc::new(crate::net::SystemHostResolver),
            firewall: Arc::new(crate::net::SystemFirewallOps),
        }
    }
}

#[cfg(all(test, unix))]
#[path = "readiness_tests.rs"]
mod tests;
