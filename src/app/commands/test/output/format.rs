use super::*;

pub(crate) fn write_results(args: &TestArgs, outputs: &[TestOutputRow]) -> crate::app::Result<()> {
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

pub(crate) fn format_tsv(outputs: &[TestOutputRow]) -> String {
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

pub(crate) fn format_csv(outputs: &[TestOutputRow]) -> String {
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

fn tsv_cell(value: Option<&str>) -> String {
    value.unwrap_or_default().replace(['\t', '\r', '\n'], " ")
}

fn csv_cell(value: Option<&str>) -> String {
    let value = value.unwrap_or_default();
    if value.contains(&[',', '"', '\r', '\n'][..]) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub(crate) fn optional_number(value: Option<u32>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

pub(crate) fn optional_float(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.2}")).unwrap_or_default()
}
