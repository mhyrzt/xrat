use super::*;

pub(crate) fn print_single_header(config: &ConfigRecord) {
    println!(
        "Testing config #{}: {}",
        config.id,
        config.name.as_deref().unwrap_or("unnamed")
    );
    println!("  Protocol: {}", config.protocol);
    println!("  Address: {}:{}", config.address, config.port);
    println!();
}

pub(crate) fn print_single_summary(output: &TestOutputRow) {
    println!();
    println!("Test completed in {:.2}s", output.elapsed_secs);

    match output.status {
        TestStatus::Skipped => println!("No tests were run"),
        TestStatus::Ok => {
            println!("OK Config is working");
            if let Some(ms) = output.real_delay_ms {
                println!("  Real delay: {ms}ms");
            }
            if let Some(mbps) = output.download_mbps {
                println!("  Download speed: {mbps:.2} Mbps");
            }
            if let Some(mbps) = output.upload_mbps {
                println!("  Upload speed: {mbps:.2} Mbps");
            }
        }
        TestStatus::Failed => {
            println!("FAIL Config failed");
            if let Some(reason) = &output.error {
                println!("  Reason: {reason}");
            }
        }
    }
}
