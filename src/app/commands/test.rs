use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::app::config::AppConfig;
use crate::app::config::defaults;
use crate::app::runtime::{AppContext, RuntimePaths};
use crate::cli::TestArgs;
use crate::db::{ConfigRecord, ConnectionTestInsert};
use crate::model::Node;
use crate::tester::{TestResult, icmp_ping, real_delay_check, tcp_check};

pub async fn run(args: &TestArgs, context: &AppContext) -> Result<(), Box<dyn std::error::Error>> {
    let settings = resolve_test_settings(args, &context.app_config, &context.runtime_paths);

    // Load config from database
    let config = context.db.get_config_by_id(args.id).await?;

    if config.is_none() {
        tracing::warn!(config_id = args.id, "config not found");
        return Ok(());
    }

    let config = config.unwrap();

    let node = node_from_record(&config)?;

    println!(
        "Testing config #{}: {}",
        config.id,
        config.name.as_deref().unwrap_or("unnamed")
    );
    println!("  Protocol: {}", config.protocol);
    println!("  Address: {}:{}", config.address, config.port);
    println!();

    let mut result = TestResult::default();
    let test_start = Instant::now();
    let ran_icmp = !args.skip_icmp;
    let ran_tcp = !args.skip_tcp;
    let mut ran_real_delay = false;

    // ICMP ping test
    if !args.skip_icmp {
        print!("Running ICMP ping... ");
        let icmp_result = icmp_ping(&config.address, settings.icmp_timeout).await;

        result.icmp_ok = icmp_result.success;
        result.icmp_ms = icmp_result.latency_ms;

        if icmp_result.success {
            println!("✓ {}ms", icmp_result.latency_ms.unwrap());
        } else {
            println!(
                "✗ {}",
                icmp_result.failure_reason.as_deref().unwrap_or("failed")
            );
            if result.failure_kind.is_none() {
                result.failure_kind = icmp_result.failure_kind;
                result.failure_reason = icmp_result.failure_reason;
            }
        }
    }

    // TCP connectivity test
    if !args.skip_tcp {
        print!("Running TCP check... ");
        let tcp_result = tcp_check(&config.address, config.port as u16, settings.tcp_timeout).await;

        result.tcp_ok = tcp_result.success;
        result.tcp_ms = tcp_result.latency_ms;

        if tcp_result.success {
            println!("✓ {}ms", tcp_result.latency_ms.unwrap());
        } else {
            println!(
                "✗ {}",
                tcp_result.failure_reason.as_deref().unwrap_or("failed")
            );
            if result.failure_kind.is_none() {
                result.failure_kind = tcp_result.failure_kind;
                result.failure_reason = tcp_result.failure_reason;
            }
        }
    }

    // Real-delay test (only if TCP succeeded or was skipped)
    if !args.skip_real_delay && (result.tcp_ok || args.skip_tcp) {
        ran_real_delay = true;
        print!("Running real-delay test... ");

        let real_delay_result = real_delay_check(
            &node,
            &settings.real_delay_url,
            &settings.xray_binary_path,
            settings.xray_startup_timeout,
            settings.real_delay_timeout,
        )
        .await;

        result.real_delay_ok = real_delay_result.success;
        result.real_delay_ms = real_delay_result.latency_ms;

        if real_delay_result.success {
            println!("✓ {}ms", real_delay_result.latency_ms.unwrap());
        } else {
            println!(
                "✗ {}",
                real_delay_result
                    .failure_reason
                    .as_deref()
                    .unwrap_or("failed")
            );
            if result.failure_kind.is_none() {
                result.failure_kind = real_delay_result.failure_kind;
                result.failure_reason = real_delay_result.failure_reason;
            }
        }
    } else if !args.skip_real_delay {
        println!("Skipping real-delay test (TCP check failed)");
    }

    let total_elapsed = test_start.elapsed();

    // Save result to database
    let failure_kind_str = result.failure_kind.as_ref().map(|k| k.as_str().to_string());

    context
        .db
        .insert_connection_test(&ConnectionTestInsert {
            config_id: config.id,
            icmp_ok: ran_icmp.then_some(result.icmp_ok),
            icmp_ms: result.icmp_ms.map(|ms| ms as i64),
            tcp_ok: ran_tcp.then_some(result.tcp_ok),
            tcp_ms: result.tcp_ms.map(|ms| ms as i64),
            real_delay_ok: ran_real_delay.then_some(result.real_delay_ok),
            real_delay_ms: result.real_delay_ms.map(|ms| ms as i64),
            failure_kind: failure_kind_str,
            failure_reason: result.failure_reason.clone(),
        })
        .await?;

    println!();
    println!("Test completed in {:.2}s", total_elapsed.as_secs_f64());

    // Print summary
    let overall_success = if ran_real_delay {
        result.real_delay_ok
    } else if ran_tcp {
        result.tcp_ok
    } else if ran_icmp {
        result.icmp_ok
    } else {
        false
    };

    if !ran_icmp && !ran_tcp && !ran_real_delay {
        println!("No tests were run");
    } else if overall_success {
        println!("✓ Config is working");
        if let Some(ms) = result.real_delay_ms {
            println!("  Real delay: {}ms", ms);
        }
    } else {
        println!("✗ Config failed");
        if let Some(reason) = &result.failure_reason {
            println!("  Reason: {}", reason);
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedTestSettings {
    real_delay_url: String,
    xray_binary_path: PathBuf,
    icmp_timeout: Duration,
    tcp_timeout: Duration,
    xray_startup_timeout: Duration,
    real_delay_timeout: Duration,
}

fn resolve_test_settings(
    args: &TestArgs,
    app_config: &AppConfig,
    runtime_paths: &RuntimePaths,
) -> ResolvedTestSettings {
    ResolvedTestSettings {
        real_delay_url: args
            .test_url
            .clone()
            .unwrap_or_else(|| app_config.testing.real_delay.url.clone()),
        xray_binary_path: resolve_engine_binary_path(app_config, runtime_paths),
        icmp_timeout: Duration::from_millis(
            args.icmp_timeout_ms
                .unwrap_or(app_config.testing.icmp.timeout),
        ),
        tcp_timeout: Duration::from_millis(
            args.tcp_timeout_ms
                .unwrap_or(app_config.testing.tcp.timeout),
        ),
        xray_startup_timeout: Duration::from_millis(defaults::DEFAULT_XRAY_STARTUP_TIMEOUT_MS),
        real_delay_timeout: Duration::from_millis(
            args.real_delay_timeout_ms
                .unwrap_or(app_config.testing.real_delay.timeout),
        ),
    }
}

fn resolve_engine_binary_path(app_config: &AppConfig, runtime_paths: &RuntimePaths) -> PathBuf {
    match app_config.runtime.engine.as_str() {
        "v2ray" => runtime_paths.v2ray_path.clone(),
        "xray" => runtime_paths.xray_path.clone(),
        other => PathBuf::from(other),
    }
}

fn node_from_record(config: &ConfigRecord) -> Result<Node, Box<dyn std::error::Error>> {
    let protocol = match config.protocol.as_str() {
        "vless" => crate::model::Protocol::Vless,
        "vmess" => crate::model::Protocol::Vmess,
        "ss" => crate::model::Protocol::Ss,
        "trojan" => crate::model::Protocol::Trojan,
        "http" => crate::model::Protocol::Http,
        "socks5" => crate::model::Protocol::Socks5,
        other => return Err(format!("unsupported protocol in database: {other}").into()),
    };

    Ok(Node {
        protocol,
        address: config.address.clone(),
        port: config.port as u16,
        username: config.username.clone(),
        uuid: config.uuid.clone(),
        password: config.password.clone(),
        method: config.method.clone(),
        network: config.network.clone(),
        tls: config.tls.clone(),
        sni: config.sni.clone(),
        host: config.host.clone(),
        path: config.path.clone(),
        name: config.name.clone(),
        raw_config: config.raw_config.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::config::{AppConfig, TestingSettings};
    use crate::cli::TestArgs;

    #[test]
    fn rebuilds_node_from_config_record() {
        let record = ConfigRecord {
            id: 1,
            subscription_id: Some(2),
            dedup_key: "key".to_string(),
            protocol: "vmess".to_string(),
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: Some("uuid-123".to_string()),
            password: None,
            method: None,
            network: "ws".to_string(),
            tls: Some("tls".to_string()),
            sni: Some("cdn.example.com".to_string()),
            host: Some("cdn.example.com".to_string()),
            path: Some("/socket".to_string()),
            name: Some("node".to_string()),
            raw_config: "vmess://payload".to_string(),
            is_active: false,
            is_enabled: true,
            is_selected: false,
            imported_at: "now".to_string(),
            created_at: "now".to_string(),
            updated_at: "now".to_string(),
        };

        let node = node_from_record(&record).expect("config record should rebuild");
        assert_eq!(node.protocol.as_str(), "vmess");
        assert_eq!(node.address, "example.com");
        assert_eq!(node.network, "ws");
        assert_eq!(node.uuid.as_deref(), Some("uuid-123"));
    }

    #[test]
    fn resolves_test_settings_from_app_config() {
        let app_config = AppConfig {
            testing: TestingSettings {
                real_delay: crate::app::config::RealDelayTestSettings {
                    url: "https://example.test/204".to_string(),
                    timeout: 12_000,
                },
                icmp: crate::app::config::TimeoutSettings { timeout: 2500 },
                tcp: crate::app::config::TimeoutSettings { timeout: 4500 },
                ..TestingSettings::default()
            },
            ..AppConfig::default()
        };
        let args = TestArgs {
            id: 1,
            skip_icmp: false,
            skip_tcp: false,
            skip_real_delay: false,
            test_url: None,
            icmp_timeout_ms: None,
            tcp_timeout_ms: None,
            real_delay_timeout_ms: None,
        };

        let runtime_paths = test_runtime_paths();
        let settings = resolve_test_settings(&args, &app_config, &runtime_paths);

        assert_eq!(settings.real_delay_url, "https://example.test/204");
        assert_eq!(settings.xray_binary_path, PathBuf::from("xray"));
        assert_eq!(settings.icmp_timeout, Duration::from_millis(2500));
        assert_eq!(settings.tcp_timeout, Duration::from_millis(4500));
        assert_eq!(settings.xray_startup_timeout, Duration::from_millis(5000));
        assert_eq!(settings.real_delay_timeout, Duration::from_millis(12_000));
    }

    #[test]
    fn cli_test_settings_override_app_config() {
        let app_config = AppConfig {
            testing: TestingSettings {
                real_delay: crate::app::config::RealDelayTestSettings {
                    url: "https://example.test/204".to_string(),
                    timeout: 12_000,
                },
                icmp: crate::app::config::TimeoutSettings { timeout: 2500 },
                tcp: crate::app::config::TimeoutSettings { timeout: 4500 },
                ..TestingSettings::default()
            },
            ..AppConfig::default()
        };
        let args = TestArgs {
            id: 1,
            skip_icmp: false,
            skip_tcp: false,
            skip_real_delay: false,
            test_url: Some("https://override.test/204".to_string()),
            icmp_timeout_ms: Some(3000),
            tcp_timeout_ms: Some(5000),
            real_delay_timeout_ms: Some(15_000),
        };

        let runtime_paths = test_runtime_paths();
        let settings = resolve_test_settings(&args, &app_config, &runtime_paths);

        assert_eq!(settings.real_delay_url, "https://override.test/204");
        assert_eq!(settings.icmp_timeout, Duration::from_millis(3000));
        assert_eq!(settings.tcp_timeout, Duration::from_millis(5000));
        assert_eq!(settings.real_delay_timeout, Duration::from_millis(15_000));
    }

    #[test]
    fn resolves_xray_binary_from_runtime_paths() {
        let app_config = AppConfig::default();
        let runtime_paths = crate::app::runtime::RuntimePaths {
            database_path: "/tmp/xrat/db.sqlite".into(),
            config_path: "/tmp/xrat/config.toml".into(),
            xray_path: "/tmp/xrat/bin/xray".into(),
            v2ray_path: "/tmp/xrat/bin/v2ray".into(),
        };

        let resolved = resolve_engine_binary_path(&app_config, &runtime_paths);

        assert_eq!(resolved, PathBuf::from("/tmp/xrat/bin/xray"));
    }

    #[test]
    fn resolves_v2ray_binary_when_engine_is_v2ray() {
        let app_config = AppConfig {
            paths: crate::app::config::PathSettings {
                xray: Some("bin/xray".into()),
                v2ray: Some("/opt/v2ray/v2ray".into()),
                ..Default::default()
            },
            runtime: crate::app::config::RuntimeSettings {
                engine: "v2ray".to_string(),
                ..Default::default()
            },
            ..AppConfig::default()
        };

        let runtime_paths = crate::app::runtime::RuntimePaths {
            database_path: "/tmp/xrat/db.sqlite".into(),
            config_path: "/tmp/xrat/config.toml".into(),
            xray_path: "/tmp/xrat/bin/xray".into(),
            v2ray_path: "/opt/v2ray/v2ray".into(),
        };

        let resolved = resolve_engine_binary_path(&app_config, &runtime_paths);

        assert_eq!(resolved, PathBuf::from("/opt/v2ray/v2ray"));
    }

    fn test_runtime_paths() -> crate::app::runtime::RuntimePaths {
        crate::app::runtime::RuntimePaths {
            database_path: "/tmp/xrat/db.sqlite".into(),
            config_path: "/tmp/xrat/config.toml".into(),
            xray_path: "xray".into(),
            v2ray_path: "v2ray".into(),
        }
    }
}
