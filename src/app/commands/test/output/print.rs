use super::*;
use crate::app::commands::output as cli_output;

pub(crate) fn print_single_header(config: &ConfigRecord) {
    println!(
        "{}",
        cli_output::format_kv(
            Some("Testing config"),
            &[
                ("id", format!("#{}", config.id)),
                ("name", cli_output::dash(config.name.as_deref())),
                ("protocol", config.protocol.clone()),
                ("address", format!("{}:{}", config.address, config.port)),
            ],
            cli_output::color_enabled(),
        )
    );
    println!();
}

pub(crate) fn print_single_summary(output: &TestOutputRow) {
    println!();
    println!(
        "{}",
        cli_output::format_kv(
            Some("Test summary"),
            &[("elapsed", format!("{:.2}s", output.elapsed_secs))],
            cli_output::color_enabled(),
        )
    );

    match output.status {
        TestStatus::Skipped => println!(
            "{}",
            cli_output::notice("No tests were run.", cli_output::color_enabled())
        ),
        TestStatus::Ok => {
            println!(
                "{}",
                cli_output::success("Config is working.", cli_output::color_enabled())
            );
            println!("{}", single_metrics(output));
        }
        TestStatus::Failed => {
            println!("FAIL Config failed");
            if let Some(reason) = &output.error {
                println!(
                    "{}",
                    cli_output::format_kv(
                        None,
                        &[("reason", reason.clone())],
                        cli_output::color_enabled(),
                    )
                );
            }
        }
    }
}

fn single_metrics(row: &TestOutputRow) -> String {
    cli_output::format_kv(
        None,
        &[
            (
                "real delay",
                row.real_delay_ms
                    .map(|value| format!("{value}ms"))
                    .unwrap_or_else(|| "-".to_string()),
            ),
            (
                "download",
                row.download_mbps
                    .map(|value| format!("{value:.2} Mbps"))
                    .unwrap_or_else(|| "-".to_string()),
            ),
            (
                "upload",
                row.upload_mbps
                    .map(|value| format!("{value:.2} Mbps"))
                    .unwrap_or_else(|| "-".to_string()),
            ),
        ],
        cli_output::color_enabled(),
    )
}
