use std::path::Path;
use std::process::{Command, Stdio};

use semver::Version;

use crate::app::{AppError, Result};

/// Minimum supported sing-box version. Newer versions are accepted so users are
/// not blocked by a hard ceiling; preflight still runs `sing-box check` with the
/// actual binary before any process starts.
const MINIMUM_VERSION: Version = Version::new(1, 13, 0);
/// Planned conformance range. Versions outside it are accepted with a warning.
const TESTED_RANGE: &str = ">=1.13.0, <1.15.0";

pub(crate) fn ensure_supported_binary(binary_path: &Path) -> Result<Version> {
    let output = Command::new(binary_path)
        .args(["version", "--name"])
        .stdin(Stdio::null())
        .output()
        .map_err(|error| {
            version_error(
                binary_path,
                None,
                &format!("could not run `version --name`: {error}"),
            )
        })?;

    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(version_error(
            binary_path,
            None,
            &format!(
                "`version --name` exited with {}; {}",
                output.status,
                if detail.is_empty() {
                    "no diagnostic output"
                } else {
                    &detail
                }
            ),
        ));
    }

    let output = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let version = parse_version_output(&output)
        .ok_or_else(|| version_error(binary_path, Some(&output), "version output is malformed"))?;
    ensure_supported_version(binary_path, &version)?;
    warn_untested_version(binary_path, &version);
    Ok(version)
}

fn parse_version_output(output: &str) -> Option<Version> {
    if output.lines().count() != 1 {
        return None;
    }
    Version::parse(output).ok()
}

fn ensure_supported_version(binary_path: &Path, version: &Version) -> Result<()> {
    if version >= &MINIMUM_VERSION {
        return Ok(());
    }
    Err(version_error(
        binary_path,
        Some(&version.to_string()),
        "version is older than the minimum supported release",
    ))
}

/// Warn when the binary is newer than (or a prerelease of) the version range
/// covered by conformance fixtures. The config may still be valid; preflight
/// validates it with the actual binary before launch.
fn warn_untested_version(binary_path: &Path, version: &Version) {
    if is_tested_version(version) {
        return;
    }
    tracing::warn!(
        binary = %binary_path.display(),
        detected = %version,
        tested_range = TESTED_RANGE,
        "sing-box version is outside the planned conformance range; managed configs are validated by preflight"
    );
}

fn is_tested_version(version: &Version) -> bool {
    version.major == 1 && matches!(version.minor, 13 | 14) && version.pre.is_empty()
}

fn version_error(binary_path: &Path, detected: Option<&str>, detail: &str) -> AppError {
    let detected = detected
        .filter(|value| !value.is_empty())
        .unwrap_or("unavailable");
    AppError::InvalidArgument(format!(
        "unsupported sing-box binary at {}: detected version {detected}; supported range is >=1.13.0; {detail}. Install sing-box v1.13.21 or newer",
        binary_path.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_name_only_version_output() {
        assert_eq!(
            parse_version_output("1.13.21"),
            Some(Version::parse("1.13.21").unwrap())
        );
        assert_eq!(
            parse_version_output("1.15.0-alpha.6"),
            Some(Version::parse("1.15.0-alpha.6").unwrap())
        );
    }

    #[test]
    fn rejects_noncanonical_version_output() {
        assert_eq!(parse_version_output("sing-box version 1.13.21"), None);
        assert_eq!(parse_version_output("v1.13.21"), None);
        assert_eq!(parse_version_output("1.13.21\nextra"), None);
        assert_eq!(parse_version_output("not-a-version"), None);
    }

    #[test]
    fn accepts_versions_at_or_above_the_minimum() {
        let binary = Path::new("/opt/xrat/bin/sing-box");
        for version in [
            "1.13.0",
            "1.13.21",
            "1.14.1",
            "1.15.0",
            "1.15.0-alpha.6",
            "2.0.0",
        ] {
            assert!(
                ensure_supported_version(binary, &Version::parse(version).unwrap()).is_ok(),
                "{version} should be accepted"
            );
        }

        let error = ensure_supported_version(binary, &Version::parse("1.12.9").unwrap())
            .expect_err("pre-1.13 should fail");
        let message = error.to_string();
        assert!(message.contains("/opt/xrat/bin/sing-box"));
        assert!(message.contains("1.12.9"));
        assert!(message.contains(">=1.13.0"));
        assert!(message.contains("Install sing-box v1.13.21 or newer"));
    }

    #[test]
    fn warns_only_outside_the_tested_range() {
        for version in ["1.13.0", "1.13.21", "1.14.0", "1.14.1"] {
            assert!(
                is_tested_version(&Version::parse(version).unwrap()),
                "{version} is inside the tested range"
            );
        }
        for version in ["1.12.9", "1.15.0", "1.15.0-alpha.6", "2.0.0"] {
            assert!(
                !is_tested_version(&Version::parse(version).unwrap()),
                "{version} is outside the tested range"
            );
        }
    }

    #[test]
    fn reports_an_unavailable_binary_with_remediation() {
        let binary = Path::new("/definitely-not-installed/sing-box");
        let error = ensure_supported_binary(binary).expect_err("missing binary should fail");
        let message = error.to_string();
        assert!(message.contains("/definitely-not-installed/sing-box"));
        assert!(message.contains("detected version unavailable"));
        assert!(message.contains(">=1.13.0"));
        assert!(message.contains("Install sing-box v1.13.21 or newer"));
    }
}
