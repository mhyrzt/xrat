use super::*;

pub(super) fn print_single_header(config: &ConfigRecord) {
    println!(
        "Testing config #{}: {}",
        config.id,
        config.name.as_deref().unwrap_or("unnamed")
    );
    println!("  Protocol: {}", config.protocol);
    println!("  Address: {}:{}", config.address, config.port);
    println!();
}

pub(super) fn print_single_summary(output: &TestOutputRow) {
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

pub(super) fn write_results(args: &TestArgs, outputs: &[TestOutputRow]) -> crate::app::Result<()> {
    let data = match args.format {
        TestFormat::Tsv => format_tsv(outputs),
        TestFormat::Csv => format_csv(outputs),
        TestFormat::Json => serde_json::to_string_pretty(outputs)?,
    };

    if let Some(path) = &args.output {
        std::fs::write(path, data)?;
    } else {
        println!("{data}");
    }

    Ok(())
}

pub(super) fn format_tsv(outputs: &[TestOutputRow]) -> String {
    let mut lines = Vec::with_capacity(outputs.len() + 1);
    lines.push("id\tname\tprotocol\taddress\tport\ticmp_ms\treal_delay_ms\tdownload_mbps\tupload_mbps\tstatus\terror".to_string());

    for output in outputs {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            output.id,
            tsv_cell(output.name.as_deref()),
            output.protocol,
            output.address,
            output.port,
            optional_number(output.icmp_ms),
            optional_number(output.real_delay_ms),
            output
                .download_mbps
                .map(|value| format!("{value:.2}"))
                .unwrap_or_default(),
            output
                .upload_mbps
                .map(|value| format!("{value:.2}"))
                .unwrap_or_default(),
            output.status.as_str(),
            tsv_cell(output.error.as_deref()),
        ));
    }

    lines.join("\n")
}

pub(super) fn format_csv(outputs: &[TestOutputRow]) -> String {
    let mut lines = Vec::with_capacity(outputs.len() + 1);
    lines.push("id,name,protocol,address,port,icmp_ms,real_delay_ms,download_mbps,upload_mbps,status,error".to_string());

    for output in outputs {
        lines.push(
            [
                output.id.to_string(),
                csv_cell(output.name.as_deref()),
                csv_cell(Some(&output.protocol)),
                csv_cell(Some(&output.address)),
                output.port.to_string(),
                optional_number(output.icmp_ms),
                optional_number(output.real_delay_ms),
                optional_float(output.download_mbps),
                optional_float(output.upload_mbps),
                output.status.as_str().to_string(),
                csv_cell(output.error.as_deref()),
            ]
            .join(","),
        );
    }

    lines.join("\n")
}

pub(super) fn tsv_cell(value: Option<&str>) -> String {
    value.unwrap_or_default().replace(['\t', '\r', '\n'], " ")
}

pub(super) fn csv_cell(value: Option<&str>) -> String {
    let value = value.unwrap_or_default();
    if value.contains(&[',', '"', '\r', '\n'][..]) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub(super) fn optional_number(value: Option<u32>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

pub(super) fn optional_float(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.2}")).unwrap_or_default()
}

pub(super) fn sort_results(outputs: &mut [TestOutputRow], sort_by: TestSortBy) {
    outputs
        .sort_by(|left, right| compare_results(left, right, sort_by).then(left.id.cmp(&right.id)));
}

pub(super) fn compare_results(
    left: &TestOutputRow,
    right: &TestOutputRow,
    sort_by: TestSortBy,
) -> Ordering {
    match sort_by {
        TestSortBy::Status => left.status.cmp(&right.status),
        TestSortBy::Icmp => compare_optional_u32(left.icmp_ms, right.icmp_ms),
        TestSortBy::RealDelay => compare_optional_u32(left.real_delay_ms, right.real_delay_ms),
        TestSortBy::DownloadSpeed => {
            compare_optional_f64_desc(left.download_mbps, right.download_mbps)
        }
        TestSortBy::Protocol => left.protocol.cmp(&right.protocol),
        TestSortBy::Address => left.address.cmp(&right.address),
    }
}

pub(super) fn compare_optional_u32(left: Option<u32>, right: Option<u32>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

pub(super) fn compare_optional_f64_desc(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => right.partial_cmp(&left).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

pub(super) fn resolve_concurrency(value: i32) -> crate::app::Result<usize> {
    if value < 0 {
        return Err(AppError::InvalidArgument(
            "test concurrency must be 0 or greater".to_string(),
        ));
    }

    if value > 0 {
        return Ok(value as usize);
    }

    let parallelism = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(1);
    Ok(parallelism.clamp(1, 8))
}
