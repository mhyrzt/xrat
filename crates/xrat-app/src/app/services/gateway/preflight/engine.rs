use super::{Check, check};

pub(super) fn inspect_tun_metadata(metadata: &std::fs::Metadata) -> Check {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        if !metadata.file_type().is_char_device() {
            return check(
                "tun_device",
                "failed",
                "/dev/net/tun exists but is not a character device.",
            );
        }
        check(
            "tun_device",
            "incomplete",
            "/dev/net/tun is a character device; runtime-owner access and TUN creation are not verified.",
        )
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        check(
            "tun_device",
            "unknown",
            "TUN device type cannot be inspected on this platform.",
        )
    }
}

pub(super) fn inspect_engine(engine: &str, path: Option<&std::path::Path>) -> Check {
    if !matches!(engine, "xray" | "sing-box") {
        return check(
            "engine",
            "failed",
            format!(
                "Configured engine '{engine}' is unsupported for the gateway MVP; select Xray or sing-box."
            ),
        );
    }
    let Some(path) = path else {
        return check(
            "engine",
            "unknown",
            format!(
                "Configured engine preference is {engine}; binary path and selected proxy backend are unavailable."
            ),
        );
    };
    let Some(path) = crate::app::tun_privileges::resolve_executable(path) else {
        return check(
            "engine",
            "incomplete",
            format!(
                "Configured {engine} binary was not found at {}; the selected proxy backend is not known.",
                path.display()
            ),
        );
    };
    let version = match engine {
        "sing-box" => xrat_engines::singbox::ensure_supported_binary(&path)
            .map(|version| version.to_string())
            .map_err(|error| error.to_string()),
        _ => {
            let spawner = std::sync::Arc::new(xrat_support::process::SystemProcessSpawner);
            let version = crate::app::services::runtime_tuning::xray_binary_version_with_spawner(
                &path, spawner,
            );
            match version {
                Some((major, minor, patch))
                    if (major, minor, patch)
                        >= crate::app::services::runtime_tuning::XRAY_TUN_MIN_VERSION =>
                {
                    Ok(format!("{major}.{minor}.{patch}"))
                }
                Some((major, minor, patch)) => {
                    let (min_major, min_minor, min_patch) =
                        crate::app::services::runtime_tuning::XRAY_TUN_MIN_VERSION;
                    Err(format!(
                        "Xray {major}.{minor}.{patch} predates the host TUN minimum {min_major}.{min_minor}.{min_patch}"
                    ))
                }
                None => Err("could not determine Xray version".into()),
            }
        }
    };
    match version {
        Ok(version) => check(
            "engine",
            "incomplete",
            format!(
                "Configured {engine} binary {} reports {version} and meets the existing host TUN version gate. The selected proxy backend, gateway configuration and packet behavior are unverified.",
                path.display()
            ),
        ),
        Err(error) => check(
            "engine",
            "incomplete",
            format!(
                "Configured {engine} binary {} did not pass the host TUN version check: {error}. The selected proxy backend is not known.",
                path.display()
            ),
        ),
    }
}
