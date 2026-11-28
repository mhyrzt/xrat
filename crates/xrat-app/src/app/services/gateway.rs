use serde::Serialize;

use crate::app::{AppError, Result};

mod firewall;
mod journal;
#[cfg(target_os = "linux")]
mod lifecycle;
#[cfg(target_os = "linux")]
pub(crate) use lifecycle::{activate, deactivate, install, recover_boot, remove};
mod preflight;
mod settings;

pub use preflight::{Report, inspect};
pub use settings::{EffectiveSettings, GatewayOverrides, ValueWithSource, resolve};
#[cfg(target_os = "linux")]
use settings::{validate_interface, validate_subnet};

#[derive(Debug, Serialize)]
pub(crate) struct ObservedGateway {
    pub installation: &'static str,
    pub protection: &'static str,
}

pub(crate) fn observe(directory: Option<&std::path::Path>) -> ObservedGateway {
    let Some(directory) = directory else {
        return ObservedGateway {
            installation: "unknown",
            protection: "unknown",
        };
    };
    match journal::load(directory) {
        Ok(None) => ObservedGateway {
            installation: "absent",
            protection: "unknown",
        },
        Err(_) => ObservedGateway {
            installation: "unknown",
            protection: "unknown",
        },
        Ok(Some(intent)) => {
            #[cfg(target_os = "linux")]
            let protection = match firewall::verify_no_flow_offload() {
                Ok(()) => match firewall::inspect_closed(&intent) {
                    Ok(true) => "closed_guard_observed",
                    Ok(false) => match lifecycle::inspect_active(directory, &intent) {
                        Ok(true) => "active_guard_observed",
                        Ok(false) => "degraded",
                        Err(_) => "unknown",
                    },
                    Err(_) => "unknown",
                },
                Err(_) => "unknown",
            };
            #[cfg(not(target_os = "linux"))]
            let protection = {
                let _ = intent;
                "unknown"
            };
            ObservedGateway {
                installation: "recorded",
                protection,
            }
        }
    }
}

pub(crate) fn validate_managed_launch(
    context: &crate::app::context::AppContext,
    raw_profile: bool,
) -> Result<Option<uuid::Uuid>> {
    let Some(intent) = journal::load(&context.runtime_paths.runtime_dir)? else {
        return Ok(None);
    };
    let runtime = &context.app_config.runtime;
    if !cfg!(target_os = "linux")
        || raw_profile
        || !matches!(runtime.engine.as_str(), "xray" | "sing-box")
        || !runtime.tun.enabled
        || runtime.tun.auto_route
        || runtime.tun.split_mode != crate::app::config::TunSplitMode::All
        || runtime.tun.kill_switch.is_enabled()
        || context.app_config.dns.fakeip.enabled
    {
        return Err(AppError::InvalidArgument(
            "gateway launch requires Linux, a generated Xray or sing-box profile, and a managed TUN with auto_route=false, split_mode=all, host kill switch disabled, and FakeIP disabled".into(),
        ));
    }
    let guard_ready = firewall::inspect_closed(&intent).map_err(|error| {
        AppError::InvalidArgument(format!(
            "gateway launch requires a verified closed forwarding guard: {error}"
        ))
    })?;
    #[cfg(target_os = "linux")]
    let guard_ready =
        guard_ready || lifecycle::inspect_active(&context.runtime_paths.runtime_dir, &intent)?;
    if !guard_ready {
        return Err(AppError::InvalidArgument(
            "gateway launch requires a verified closed forwarding guard".into(),
        ));
    }
    #[cfg(target_os = "linux")]
    lifecycle::preflight_launch_routes(&context.runtime_paths.runtime_dir, &intent).map_err(
        |error| AppError::InvalidArgument(format!("gateway route preflight failed: {error}")),
    )?;
    Ok(Some(intent.installation_id))
}
