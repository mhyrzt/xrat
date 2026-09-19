use std::path::Path;
use std::time::Duration;

use crate::model::Node;
use crate::singbox::{SingboxProbeError, SingboxProbeProcess, generate_singbox_probe_config};
use crate::xray::{
    XrayGenOptions, XrayProcess, XrayProcessError, generate_probe_config_with_options,
};

use super::FailureKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeEngineKind {
    Xray,
    Singbox,
}

pub enum ProbeProcess {
    Xray(XrayProcess),
    Singbox(SingboxProbeProcess),
}

impl ProbeProcess {
    /// Spawn the engine-specific probe process for `node`. Configuration or
    /// process failures are returned as an already-classified
    /// `(FailureKind, reason)` pair so every probe stage reports them the same
    /// way.
    pub async fn spawn(
        node: &Node,
        local_port: u16,
        engine: ProbeEngineKind,
        binary_path: &Path,
        gen_options: &XrayGenOptions,
        startup_timeout: Duration,
    ) -> Result<Self, (FailureKind, String)> {
        match engine {
            ProbeEngineKind::Xray => {
                let config = generate_probe_config_with_options(node, local_port, gen_options)
                    .map_err(|error| {
                        (
                            FailureKind::Process,
                            format!("Failed to generate config: {error}"),
                        )
                    })?;
                XrayProcess::spawn_with_binary(binary_path, &config, startup_timeout)
                    .await
                    .map(Self::Xray)
                    .map_err(|error| classify_xray(&error))
            }
            ProbeEngineKind::Singbox => {
                let config = generate_singbox_probe_config(node, local_port).map_err(|error| {
                    (
                        FailureKind::Process,
                        format!("Failed to generate config: {error}"),
                    )
                })?;
                SingboxProbeProcess::spawn_with_binary(
                    binary_path,
                    &config,
                    local_port,
                    startup_timeout,
                )
                .await
                .map(Self::Singbox)
                .map_err(|error| classify_singbox(&error))
            }
        }
    }

    pub fn local_port(&self) -> u16 {
        match self {
            Self::Xray(process) => process.local_port(),
            Self::Singbox(process) => process.local_port(),
        }
    }

    pub fn kill(self) -> Result<(), std::io::Error> {
        match self {
            Self::Xray(process) => process.kill(),
            Self::Singbox(process) => process.kill(),
        }
    }
}

fn classify_xray(error: &XrayProcessError) -> (FailureKind, String) {
    match error {
        XrayProcessError::SpawnError(_) => (
            FailureKind::Process,
            format!("Failed to spawn xray: {error}"),
        ),
        XrayProcessError::StartupTimeout => {
            (FailureKind::Timeout, "Xray startup timeout".to_string())
        }
        XrayProcessError::ProcessExited(stderr) => (
            FailureKind::Process,
            format!("Xray process exited unexpectedly: {stderr}"),
        ),
        XrayProcessError::PortNotReady(_) => (
            FailureKind::Process,
            format!("Xray port not ready: {error}"),
        ),
        _ => (FailureKind::Process, format!("Xray error: {error}")),
    }
}

fn classify_singbox(error: &SingboxProbeError) -> (FailureKind, String) {
    match error {
        SingboxProbeError::Spawn(_) => (
            FailureKind::Process,
            format!("Failed to spawn sing-box: {error}"),
        ),
        SingboxProbeError::ProcessExited(stderr) => (
            FailureKind::Process,
            format!("sing-box process exited unexpectedly: {stderr}"),
        ),
        SingboxProbeError::PortNotReady(_) => (
            FailureKind::Process,
            format!("sing-box port not ready: {error}"),
        ),
        _ => (FailureKind::Process, format!("sing-box error: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Protocol;
    use std::path::Path;

    fn node(protocol: Protocol) -> Node {
        Node {
            protocol,
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: Some("00000000-0000-0000-0000-000000000001".to_string()),
            password: Some("secret".to_string()),
            method: None,
            network: "tcp".to_string(),
            tls: Some("tls".to_string()),
            sni: None,
            host: None,
            path: None,
            name: None,
            extensions: None,
            raw_config: "vless://00000000-0000-0000-0000-000000000001@example.com:443?security=tls"
                .to_string(),
        }
    }

    #[tokio::test]
    async fn singbox_probe_generates_config_and_reports_spawn_failure() {
        let error = ProbeProcess::spawn(
            &node(Protocol::Vless),
            1080,
            ProbeEngineKind::Singbox,
            Path::new("/definitely-not-installed/sing-box"),
            &XrayGenOptions::default(),
            Duration::from_millis(100),
        )
        .await
        .err()
        .expect("missing sing-box binary must fail to spawn");

        assert!(matches!(error.0, FailureKind::Process));
        assert!(error.1.contains("Failed to spawn sing-box"));
    }

    #[tokio::test]
    async fn singbox_probe_rejects_unrepresentable_config_before_spawning() {
        let mut invalid = node(Protocol::Vless);
        invalid.uuid = None;
        let error = ProbeProcess::spawn(
            &invalid,
            1080,
            ProbeEngineKind::Singbox,
            Path::new("/definitely-not-installed/sing-box"),
            &XrayGenOptions::default(),
            Duration::from_millis(100),
        )
        .await
        .err()
        .expect("invalid node must fail config generation");

        assert!(error.1.contains("Failed to generate config"));
        assert!(error.1.contains("VLESS requires UUID"));
    }

    #[test]
    fn classifies_singbox_process_errors() {
        let (kind, reason) =
            classify_singbox(&SingboxProbeError::ProcessExited("boom".to_string()));
        assert!(matches!(kind, FailureKind::Process));
        assert!(reason.contains("sing-box process exited unexpectedly: boom"));
    }
}
