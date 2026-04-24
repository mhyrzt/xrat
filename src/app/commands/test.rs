use std::time::Instant;

use crate::cli::TestArgs;
use crate::db::{ConnectionTestInsert, Database};
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

    // Parse the node from raw_config
    let node: Node = serde_json::from_str(&config.raw_config)?;

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
        icmp_ok: Some(result.icmp_ok),
        icmp_ms: result.icmp_ms.map(|ms| ms as i64),
        tcp_ok: Some(result.tcp_ok),
        tcp_ms: result.tcp_ms.map(|ms| ms as i64),
        real_delay_ok: Some(result.real_delay_ok),
        real_delay_ms: result.real_delay_ms.map(|ms| ms as i64),
        failure_kind: failure_kind_str,
        failure_reason: result.failure_reason.clone(),
    })
    .await?;

    println!();
    println!("Test completed in {:.2}s", total_elapsed.as_secs_f64());

    // Print summary
    let overall_success = result.real_delay_ok || (result.tcp_ok && args.skip_real_delay);
    if overall_success {
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
