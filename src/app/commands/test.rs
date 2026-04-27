use std::time::Instant;

use crate::cli::TestArgs;
use crate::db::{ConfigRecord, ConnectionTestInsert, Database};
use crate::model::Node;
use crate::tester::real_delay::DEFAULT_TEST_URL;
use crate::tester::{
    DEFAULT_ICMP_TIMEOUT, DEFAULT_REAL_DELAY_TIMEOUT, DEFAULT_TCP_TIMEOUT,
    DEFAULT_XRAY_STARTUP_TIMEOUT, TestResult, icmp_ping, real_delay_check, tcp_check,
};

pub async fn run(args: &TestArgs, db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    // Load config from database
    let config = db.get_config_by_id(args.id).await?;

    if config.is_none() {
        eprintln!("Config with id {} not found", args.id);
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
        let icmp_result = icmp_ping(&config.address, DEFAULT_ICMP_TIMEOUT).await;

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
        let tcp_result = tcp_check(&config.address, config.port as u16, DEFAULT_TCP_TIMEOUT).await;

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
        let test_url = args.test_url.as_deref().unwrap_or(DEFAULT_TEST_URL);

        let real_delay_result = real_delay_check(
            &node,
            test_url,
            DEFAULT_XRAY_STARTUP_TIMEOUT,
            DEFAULT_REAL_DELAY_TIMEOUT,
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

    db.insert_connection_test(&ConnectionTestInsert {
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
}
