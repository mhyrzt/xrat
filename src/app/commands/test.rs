use std::cmp::Ordering;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use tokio::task::JoinSet;

use crate::app::AppError;
use crate::app::config::defaults;
use crate::app::config::{AppConfig, ConnectionTestStage, TestFailurePolicy};
use crate::app::runtime::{AppContext, RuntimePaths};
use crate::cli::{TestArgs, TestFormat, TestSortBy};
#[cfg(test)]
use crate::db::DatabaseConnectionConfig;
use crate::db::{ConfigRecord, ConnectionTestInsert, Database};
use crate::model::Node;
use crate::tester::{
    FailureKind, TestResult, download_speed_check, icmp_ping, real_delay_check, tcp_check,
};

pub async fn run(args: &TestArgs, context: &AppContext) -> crate::app::Result<()> {
    let settings = resolve_test_settings(args, &context.app_config, &context.runtime_paths)?;

    if let Some(config_id) = args.id {
        run_single(args, context, settings, config_id).await
    } else {
        run_bulk(args, context, settings).await
    }
}

async fn run_single(
    _args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    config_id: i64,
) -> crate::app::Result<()> {
    let config = context.db.get_config_by_id(config_id).await?;

    let Some(config) = config else {
        tracing::warn!(config_id, "config not found");
        return Ok(());
    };

    print_single_header(&config);
    let output = test_and_record_config(context.db.clone(), config, settings, true).await?;
    print_single_summary(&output);

    Ok(())
}

async fn run_bulk(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
) -> crate::app::Result<()> {
    let configs = context.db.list_configs(&args.config_filter()).await?;

    if configs.is_empty() {
        tracing::info!("no configs found for requested test filters");
        write_results(args, &[])?;
        return Ok(());
    }

    let total = configs.len();
    let concurrency = resolve_concurrency(settings.concurrency)?;
    let progress = bulk_progress_bar(total, !args.no_progress);
    let mut next_config = configs.into_iter();
    let mut join_set = JoinSet::new();
    let mut completed = 0usize;
    let mut failed = 0usize;
    let mut outputs = Vec::with_capacity(total);

    for _ in 0..concurrency {
        let Some(config) = next_config.next() else {
            break;
        };
        spawn_config_test(&mut join_set, context.db.clone(), config, settings.clone());
    }

    while let Some(joined) = join_set.join_next().await {
        let output = joined??;
        completed += 1;
        if output.status != TestStatus::Ok {
            failed += 1;
        }
        outputs.push(output);
        update_bulk_progress(&progress, completed, failed);

        if let Some(config) = next_config.next() {
            spawn_config_test(&mut join_set, context.db.clone(), config, settings.clone());
        }
    }
    finish_bulk_progress(progress, completed, failed);

    sort_results(&mut outputs, args.sort_by);
    write_results(args, &outputs)?;

    Ok(())
}

fn bulk_progress_bar(total: usize, enabled: bool) -> Option<ProgressBar> {
    if !enabled {
        return None;
    }

    let progress = ProgressBar::new(total as u64);
    let style = ProgressStyle::with_template(
        "{spinner:.green} testing [{bar:32.cyan/blue}] {pos}/{len} failed:{msg}",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar())
    .progress_chars("=>-");
    progress.set_style(style);
    progress.set_message("0");

    Some(progress)
}

fn update_bulk_progress(progress: &Option<ProgressBar>, completed: usize, failed: usize) {
    if let Some(progress) = progress {
        progress.set_position(completed as u64);
        progress.set_message(failed.to_string());
    }
}

fn finish_bulk_progress(progress: Option<ProgressBar>, completed: usize, failed: usize) {
    if let Some(progress) = progress {
        progress.finish_with_message(format!("done: {completed} tested, {failed} failed"));
    }
}

fn spawn_config_test(
    join_set: &mut JoinSet<crate::app::Result<TestOutputRow>>,
    db: Database,
    config: ConfigRecord,
    settings: ResolvedTestSettings,
) {
    join_set.spawn(async move { test_and_record_config(db, config, settings, false).await });
}

async fn test_and_record_config(
    db: Database,
    config: ConfigRecord,
    settings: ResolvedTestSettings,
    print_progress: bool,
) -> crate::app::Result<TestOutputRow> {
    let node = node_from_record(&config)?;
    let mut result = TestResult::default();
    let test_start = Instant::now();
    let mut ran_icmp = false;
    let mut ran_tcp = false;
    let mut ran_real_delay = false;
    let mut ran_download = false;

    for stage in &settings.stage_order {
        match stage {
            ConnectionTestStage::Icmp if settings.run_icmp => {
                ran_icmp = true;
                run_icmp_stage(&config, &settings, &mut result, print_progress).await?;
                if !result.icmp_ok && settings.failure_policy.halts_after_failure() {
                    break;
                }
            }
            ConnectionTestStage::RealDelay if settings.run_real_delay => {
                if settings.run_tcp && !ran_tcp {
                    ran_tcp = true;
                    run_tcp_gate(&config, &settings, &mut result, print_progress).await?;
                }

                if result.tcp_ok || !settings.run_tcp {
                    ran_real_delay = true;
                    run_real_delay_stage(&node, &settings, &mut result, print_progress).await?;
                    if !result.real_delay_ok && settings.failure_policy.halts_after_failure() {
                        break;
                    }
                } else if print_progress {
                    println!("Skipping real-delay test (TCP check failed)");
                }

                if settings.run_tcp
                    && !result.tcp_ok
                    && settings.failure_policy.halts_after_failure()
                {
                    break;
                }
            }
            ConnectionTestStage::Download if settings.run_download => {
                ran_download = true;
                run_download_stage(&node, &settings, &mut result, print_progress).await?;
                if !result.download_ok && settings.failure_policy.halts_after_failure() {
                    break;
                }
            }
            _ => {}
        }
    }

    let elapsed = test_start.elapsed();
    let output = TestOutputRow::from_parts(
        &config,
        &result,
        ran_icmp,
        ran_tcp,
        ran_real_delay,
        ran_download,
        elapsed,
    );
    db.insert_connection_test(&output.connection_test_insert())
        .await?;

    Ok(output)
}

async fn run_icmp_stage(
    config: &ConfigRecord,
    settings: &ResolvedTestSettings,
    result: &mut TestResult,
    print_progress: bool,
) -> crate::app::Result<()> {
    if print_progress {
        print!("Running ICMP ping... ");
        std::io::stdout().flush()?;
    }
    let icmp_result = icmp_ping(&config.address, settings.icmp_timeout).await;
    let failure_reason = icmp_result.failure_reason.clone();

    result.icmp_ok = icmp_result.success;
    result.icmp_ms = icmp_result.latency_ms;
    merge_failure(result, icmp_result.failure_kind, icmp_result.failure_reason);

    if print_progress {
        print_stage_result(
            icmp_result.success,
            icmp_result.latency_ms,
            failure_reason.as_deref(),
        );
    }

    Ok(())
}

async fn run_tcp_gate(
    config: &ConfigRecord,
    settings: &ResolvedTestSettings,
    result: &mut TestResult,
    print_progress: bool,
) -> crate::app::Result<()> {
    if print_progress {
        print!("Running TCP check... ");
        std::io::stdout().flush()?;
    }
    let tcp_result = tcp_check(&config.address, config.port as u16, settings.tcp_timeout).await;
    let failure_reason = tcp_result.failure_reason.clone();

    result.tcp_ok = tcp_result.success;
    result.tcp_ms = tcp_result.latency_ms;
    merge_failure(result, tcp_result.failure_kind, tcp_result.failure_reason);

    if print_progress {
        print_stage_result(
            tcp_result.success,
            tcp_result.latency_ms,
            failure_reason.as_deref(),
        );
    }

    Ok(())
}

async fn run_real_delay_stage(
    node: &Node,
    settings: &ResolvedTestSettings,
    result: &mut TestResult,
    print_progress: bool,
) -> crate::app::Result<()> {
    if print_progress {
        print!("Running real-delay test... ");
        std::io::stdout().flush()?;
    }

    let real_delay_result = real_delay_check(
        node,
        &settings.real_delay_url,
        &settings.xray_binary_path,
        settings.xray_startup_timeout,
        settings.real_delay_timeout,
    )
    .await;
    let failure_reason = real_delay_result.failure_reason.clone();

    result.real_delay_ok = real_delay_result.success;
    result.real_delay_ms = real_delay_result.latency_ms;
    merge_failure(
        result,
        real_delay_result.failure_kind,
        real_delay_result.failure_reason,
    );

    if print_progress {
        print_stage_result(
            real_delay_result.success,
            real_delay_result.latency_ms,
            failure_reason.as_deref(),
        );
    }

    Ok(())
}

async fn run_download_stage(
    node: &Node,
    settings: &ResolvedTestSettings,
    result: &mut TestResult,
    print_progress: bool,
) -> crate::app::Result<()> {
    if print_progress {
        print!("Running download speed test... ");
        std::io::stdout().flush()?;
    }

    let download_result = download_speed_check(
        node,
        &settings.download_url,
        &settings.xray_binary_path,
        settings.xray_startup_timeout,
        settings.download_timeout,
    )
    .await;
    let failure_reason = download_result.failure_reason.clone();

    result.download_ok = download_result.success;
    result.download_mbps = download_result.mbps;
    merge_failure(
        result,
        download_result.failure_kind,
        download_result.failure_reason,
    );

    if print_progress {
        print_download_result(
            download_result.success,
            download_result.mbps,
            failure_reason.as_deref(),
        );
    }

    Ok(())
}

fn merge_failure(
    result: &mut TestResult,
    failure_kind: Option<FailureKind>,
    failure_reason: Option<String>,
) {
    if !matches!(result.failure_kind, None) {
        return;
    }

    result.failure_kind = failure_kind;
    result.failure_reason = failure_reason;
}

fn print_download_result(success: bool, mbps: Option<f64>, failure_reason: Option<&str>) {
    if success {
        println!("OK {:.2} Mbps", mbps.unwrap_or_default());
    } else {
        println!("FAIL {}", failure_reason.unwrap_or("failed"));
    }
}

fn print_stage_result(success: bool, latency_ms: Option<u32>, failure_reason: Option<&str>) {
    if success {
        println!("OK {}ms", latency_ms.unwrap_or_default());
    } else {
        println!("FAIL {}", failure_reason.unwrap_or("failed"));
    }
}

fn print_single_header(config: &ConfigRecord) {
    println!(
        "Testing config #{}: {}",
        config.id,
        config.name.as_deref().unwrap_or("unnamed")
    );
    println!("  Protocol: {}", config.protocol);
    println!("  Address: {}:{}", config.address, config.port);
    println!();
}

fn print_single_summary(output: &TestOutputRow) {
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
        }
        TestStatus::Failed => {
            println!("FAIL Config failed");
            if let Some(reason) = &output.error {
                println!("  Reason: {reason}");
            }
        }
    }
}

fn write_results(args: &TestArgs, outputs: &[TestOutputRow]) -> crate::app::Result<()> {
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

fn format_tsv(outputs: &[TestOutputRow]) -> String {
    let mut lines = Vec::with_capacity(outputs.len() + 1);
    lines.push(
        "id\tname\tprotocol\taddress\tport\ticmp_ms\treal_delay_ms\tdownload_mbps\tstatus\terror"
            .to_string(),
    );

    for output in outputs {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
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
            output.status.as_str(),
            tsv_cell(output.error.as_deref()),
        ));
    }

    lines.join("\n")
}

fn format_csv(outputs: &[TestOutputRow]) -> String {
    let mut lines = Vec::with_capacity(outputs.len() + 1);
    lines.push(
        "id,name,protocol,address,port,icmp_ms,real_delay_ms,download_mbps,status,error"
            .to_string(),
    );

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

fn optional_number(value: Option<u32>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn optional_float(value: Option<f64>) -> String {
    value.map(|value| format!("{value:.2}")).unwrap_or_default()
}

fn sort_results(outputs: &mut [TestOutputRow], sort_by: TestSortBy) {
    outputs
        .sort_by(|left, right| compare_results(left, right, sort_by).then(left.id.cmp(&right.id)));
}

fn compare_results(left: &TestOutputRow, right: &TestOutputRow, sort_by: TestSortBy) -> Ordering {
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

fn compare_optional_u32(left: Option<u32>, right: Option<u32>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_optional_f64_desc(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => right.partial_cmp(&left).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn resolve_concurrency(value: i32) -> crate::app::Result<usize> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedTestSettings {
    stage_order: Vec<ConnectionTestStage>,
    failure_policy: TestFailurePolicy,
    real_delay_url: String,
    download_url: String,
    xray_binary_path: PathBuf,
    icmp_timeout: Duration,
    tcp_timeout: Duration,
    xray_startup_timeout: Duration,
    real_delay_timeout: Duration,
    download_timeout: Duration,
    run_icmp: bool,
    run_tcp: bool,
    run_real_delay: bool,
    run_download: bool,
    concurrency: i32,
}

fn resolve_test_settings(
    args: &TestArgs,
    app_config: &AppConfig,
    runtime_paths: &RuntimePaths,
) -> crate::app::Result<ResolvedTestSettings> {
    let concurrency = args.concurrency.unwrap_or(app_config.testing.concurrency);
    if concurrency < 0 {
        return Err(AppError::InvalidArgument(
            "test concurrency must be 0 or greater".to_string(),
        ));
    }
    validate_test_stage_order(&app_config.testing.order)?;

    Ok(ResolvedTestSettings {
        stage_order: app_config.testing.order.clone(),
        failure_policy: app_config.testing.failure_policy,
        real_delay_url: args
            .test_url
            .clone()
            .unwrap_or_else(|| app_config.testing.real_delay.url.clone()),
        download_url: args
            .download_url
            .clone()
            .unwrap_or_else(|| app_config.testing.download.url.clone()),
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
        download_timeout: Duration::from_millis(
            args.download_timeout_ms
                .unwrap_or(app_config.testing.download.timeout),
        ),
        run_icmp: app_config.testing.icmp.enabled && !args.skip_icmp,
        run_tcp: app_config.testing.tcp.enabled && !args.skip_tcp,
        run_real_delay: app_config.testing.real_delay.enabled && !args.skip_real_delay,
        run_download: app_config.testing.download.enabled && !args.skip_download,
        concurrency,
    })
}

fn validate_test_stage_order(order: &[ConnectionTestStage]) -> crate::app::Result<()> {
    let mut seen = Vec::with_capacity(order.len());
    for stage in order {
        if seen.contains(stage) {
            return Err(AppError::InvalidArgument(format!(
                "duplicate test stage in [testing].order: {}",
                test_stage_name(*stage)
            )));
        }
        seen.push(*stage);
    }

    Ok(())
}

fn test_stage_name(stage: ConnectionTestStage) -> &'static str {
    match stage {
        ConnectionTestStage::Icmp => "icmp",
        ConnectionTestStage::RealDelay => "real_delay",
        ConnectionTestStage::Download => "download",
    }
}

impl TestFailurePolicy {
    fn halts_after_failure(self) -> bool {
        matches!(self, Self::SkipRemaining | Self::MarkFailed)
    }
}

fn resolve_engine_binary_path(app_config: &AppConfig, runtime_paths: &RuntimePaths) -> PathBuf {
    match app_config.runtime.engine.as_str() {
        "v2ray" => runtime_paths.v2ray_path.clone(),
        "xray" => runtime_paths.xray_path.clone(),
        other => PathBuf::from(other),
    }
}

#[derive(Clone, Debug, Serialize)]
struct TestOutputRow {
    id: i64,
    name: Option<String>,
    protocol: String,
    address: String,
    port: i64,
    icmp_ms: Option<u32>,
    real_delay_ms: Option<u32>,
    download_mbps: Option<f64>,
    status: TestStatus,
    error: Option<String>,
    #[serde(skip_serializing)]
    tcp_ms: Option<u32>,
    #[serde(skip_serializing)]
    ran_icmp: bool,
    #[serde(skip_serializing)]
    ran_tcp: bool,
    #[serde(skip_serializing)]
    ran_real_delay: bool,
    #[serde(skip_serializing)]
    icmp_ok: bool,
    #[serde(skip_serializing)]
    tcp_ok: bool,
    #[serde(skip_serializing)]
    real_delay_ok: bool,
    #[serde(skip_serializing)]
    failure_kind: Option<String>,
    #[serde(skip_serializing)]
    elapsed_secs: f64,
}

impl TestOutputRow {
    fn from_parts(
        config: &ConfigRecord,
        result: &TestResult,
        ran_icmp: bool,
        ran_tcp: bool,
        ran_real_delay: bool,
        ran_download: bool,
        elapsed: Duration,
    ) -> Self {
        let status = overall_status(result, ran_icmp, ran_tcp, ran_real_delay, ran_download);

        Self {
            id: config.id,
            name: config.name.clone(),
            protocol: config.protocol.clone(),
            address: config.address.clone(),
            port: config.port,
            icmp_ms: result.icmp_ms,
            real_delay_ms: result.real_delay_ms,
            download_mbps: result.download_mbps,
            status,
            error: result.failure_reason.clone(),
            tcp_ms: result.tcp_ms,
            ran_icmp,
            ran_tcp,
            ran_real_delay,
            icmp_ok: result.icmp_ok,
            tcp_ok: result.tcp_ok,
            real_delay_ok: result.real_delay_ok,
            failure_kind: result
                .failure_kind
                .as_ref()
                .map(|kind| kind.as_str().to_string()),
            elapsed_secs: elapsed.as_secs_f64(),
        }
    }

    fn connection_test_insert(&self) -> ConnectionTestInsert {
        ConnectionTestInsert {
            config_id: self.id,
            icmp_ok: self.ran_icmp.then_some(self.icmp_ok),
            icmp_ms: self.icmp_ms.map(i64::from),
            tcp_ok: self.ran_tcp.then_some(self.tcp_ok),
            tcp_ms: self.tcp_ms.map(i64::from),
            real_delay_ok: self.ran_real_delay.then_some(self.real_delay_ok),
            real_delay_ms: self.real_delay_ms.map(i64::from),
            download_mbps: self.download_mbps,
            failure_kind: self.failure_kind.clone(),
            failure_reason: self.error.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
enum TestStatus {
    Ok,
    Failed,
    Skipped,
}

impl TestStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

fn overall_status(
    result: &TestResult,
    ran_icmp: bool,
    ran_tcp: bool,
    ran_real_delay: bool,
    ran_download: bool,
) -> TestStatus {
    if !ran_icmp && !ran_tcp && !ran_real_delay && !ran_download {
        return TestStatus::Skipped;
    }

    let success = if ran_download {
        result.download_ok
    } else if ran_real_delay {
        result.real_delay_ok
    } else if ran_tcp {
        result.tcp_ok
    } else {
        result.icmp_ok
    };

    if success {
        TestStatus::Ok
    } else {
        TestStatus::Failed
    }
}

fn node_from_record(config: &ConfigRecord) -> crate::app::Result<Node> {
    let protocol = match config.protocol.as_str() {
        "vless" => crate::model::Protocol::Vless,
        "vmess" => crate::model::Protocol::Vmess,
        "ss" => crate::model::Protocol::Ss,
        "trojan" => crate::model::Protocol::Trojan,
        "http" => crate::model::Protocol::Http,
        "socks5" => crate::model::Protocol::Socks5,
        other => return Err(AppError::UnsupportedProtocol(other.to_string())),
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
                    enabled: true,
                    url: "https://example.test/204".to_string(),
                    timeout: 12_000,
                },
                download: crate::app::config::DownloadTestSettings {
                    enabled: true,
                    url: "https://example.test/10mb.test".to_string(),
                    timeout: 40_000,
                },
                icmp: crate::app::config::IcmpTestSettings {
                    enabled: true,
                    attempts: 3,
                    timeout: 2500,
                },
                tcp: crate::app::config::TcpTestSettings {
                    enabled: true,
                    timeout: 4500,
                },
                ..TestingSettings::default()
            },
            ..AppConfig::default()
        };
        let args = test_args(Some(1));

        let runtime_paths = test_runtime_paths();
        let settings = resolve_test_settings(&args, &app_config, &runtime_paths).expect("settings");

        assert_eq!(settings.real_delay_url, "https://example.test/204");
        assert_eq!(settings.download_url, "https://example.test/10mb.test");
        assert_eq!(settings.xray_binary_path, PathBuf::from("xray"));
        assert_eq!(settings.icmp_timeout, Duration::from_millis(2500));
        assert_eq!(settings.tcp_timeout, Duration::from_millis(4500));
        assert_eq!(settings.xray_startup_timeout, Duration::from_millis(5000));
        assert_eq!(settings.real_delay_timeout, Duration::from_millis(12_000));
        assert_eq!(settings.download_timeout, Duration::from_millis(40_000));
        assert_eq!(
            settings.stage_order,
            vec![
                ConnectionTestStage::Icmp,
                ConnectionTestStage::RealDelay,
                ConnectionTestStage::Download,
            ]
        );
        assert_eq!(settings.failure_policy, TestFailurePolicy::Continue);
    }

    #[test]
    fn cli_test_settings_override_app_config() {
        let app_config = AppConfig {
            testing: TestingSettings {
                real_delay: crate::app::config::RealDelayTestSettings {
                    enabled: true,
                    url: "https://example.test/204".to_string(),
                    timeout: 12_000,
                },
                icmp: crate::app::config::IcmpTestSettings {
                    enabled: true,
                    attempts: 3,
                    timeout: 2500,
                },
                tcp: crate::app::config::TcpTestSettings {
                    enabled: true,
                    timeout: 4500,
                },
                ..TestingSettings::default()
            },
            ..AppConfig::default()
        };
        let args = TestArgs {
            test_url: Some("https://override.test/204".to_string()),
            download_url: Some("https://override.test/10mb.test".to_string()),
            icmp_timeout_ms: Some(3000),
            tcp_timeout_ms: Some(5000),
            real_delay_timeout_ms: Some(15_000),
            download_timeout_ms: Some(45_000),
            ..test_args(Some(1))
        };

        let runtime_paths = test_runtime_paths();
        let settings = resolve_test_settings(&args, &app_config, &runtime_paths).expect("settings");

        assert_eq!(settings.real_delay_url, "https://override.test/204");
        assert_eq!(settings.download_url, "https://override.test/10mb.test");
        assert_eq!(settings.icmp_timeout, Duration::from_millis(3000));
        assert_eq!(settings.tcp_timeout, Duration::from_millis(5000));
        assert_eq!(settings.real_delay_timeout, Duration::from_millis(15_000));
        assert_eq!(settings.download_timeout, Duration::from_millis(45_000));
    }

    #[test]
    fn resolves_custom_test_stage_order() {
        let app_config = AppConfig {
            testing: TestingSettings {
                order: vec![ConnectionTestStage::RealDelay, ConnectionTestStage::Icmp],
                ..TestingSettings::default()
            },
            ..AppConfig::default()
        };
        let runtime_paths = test_runtime_paths();

        let settings = resolve_test_settings(&test_args(Some(1)), &app_config, &runtime_paths)
            .expect("settings");

        assert_eq!(
            settings.stage_order,
            vec![ConnectionTestStage::RealDelay, ConnectionTestStage::Icmp]
        );
    }

    #[test]
    fn rejects_duplicate_test_stage_order_entries() {
        let app_config = AppConfig {
            testing: TestingSettings {
                order: vec![ConnectionTestStage::Icmp, ConnectionTestStage::Icmp],
                ..TestingSettings::default()
            },
            ..AppConfig::default()
        };
        let runtime_paths = test_runtime_paths();

        let error = resolve_test_settings(&test_args(Some(1)), &app_config, &runtime_paths)
            .expect_err("duplicate stage should fail");

        assert!(error.to_string().contains("duplicate test stage"));
    }

    #[test]
    fn resolves_configured_failure_policy() {
        let app_config = AppConfig {
            testing: TestingSettings {
                failure_policy: TestFailurePolicy::SkipRemaining,
                ..TestingSettings::default()
            },
            ..AppConfig::default()
        };
        let runtime_paths = test_runtime_paths();

        let settings = resolve_test_settings(&test_args(Some(1)), &app_config, &runtime_paths)
            .expect("settings");

        assert_eq!(settings.failure_policy, TestFailurePolicy::SkipRemaining);
        assert!(settings.failure_policy.halts_after_failure());
        assert!(TestFailurePolicy::MarkFailed.halts_after_failure());
        assert!(!TestFailurePolicy::Continue.halts_after_failure());
    }

    #[test]
    fn formats_csv_results_with_download_speed() {
        let output = TestOutputRow {
            id: 7,
            name: Some("node, one".to_string()),
            protocol: "vless".to_string(),
            address: "example.com".to_string(),
            port: 443,
            icmp_ms: Some(12),
            real_delay_ms: Some(123),
            download_mbps: Some(45.678),
            status: TestStatus::Ok,
            error: None,
            tcp_ms: Some(25),
            ran_icmp: true,
            ran_tcp: true,
            ran_real_delay: true,
            icmp_ok: true,
            tcp_ok: true,
            real_delay_ok: true,
            failure_kind: None,
            elapsed_secs: 1.0,
        };

        let csv = format_csv(&[output]);

        assert!(csv.starts_with(
            "id,name,protocol,address,port,icmp_ms,real_delay_ms,download_mbps,status,error\n"
        ));
        assert!(csv.contains("7,\"node, one\",vless,example.com,443,12,123,45.68,ok,"));
    }

    #[test]
    fn resolves_configured_test_concurrency() {
        assert_eq!(resolve_concurrency(1).expect("positive concurrency"), 1);
        assert_eq!(resolve_concurrency(16).expect("positive concurrency"), 16);
        assert!(resolve_concurrency(-1).is_err());
        assert!(resolve_concurrency(0).expect("auto concurrency") >= 1);
    }

    #[test]
    fn resolves_xray_binary_from_runtime_paths() {
        let app_config = AppConfig::default();
        let runtime_paths = crate::app::runtime::RuntimePaths {
            database_config: DatabaseConnectionConfig::Sqlite {
                path: "/tmp/xrat/db.sqlite".into(),
            },
            database_path: "/tmp/xrat/db.sqlite".into(),
            database_label: "/tmp/xrat/db.sqlite".to_string(),
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
            database_config: DatabaseConnectionConfig::Sqlite {
                path: "/tmp/xrat/db.sqlite".into(),
            },
            database_path: "/tmp/xrat/db.sqlite".into(),
            database_label: "/tmp/xrat/db.sqlite".to_string(),
            config_path: "/tmp/xrat/config.toml".into(),
            xray_path: "/tmp/xrat/bin/xray".into(),
            v2ray_path: "/opt/v2ray/v2ray".into(),
        };

        let resolved = resolve_engine_binary_path(&app_config, &runtime_paths);

        assert_eq!(resolved, PathBuf::from("/opt/v2ray/v2ray"));
    }

    fn test_runtime_paths() -> crate::app::runtime::RuntimePaths {
        crate::app::runtime::RuntimePaths {
            database_config: DatabaseConnectionConfig::Sqlite {
                path: "/tmp/xrat/db.sqlite".into(),
            },
            database_path: "/tmp/xrat/db.sqlite".into(),
            database_label: "/tmp/xrat/db.sqlite".to_string(),
            config_path: "/tmp/xrat/config.toml".into(),
            xray_path: "xray".into(),
            v2ray_path: "v2ray".into(),
        }
    }

    fn test_args(id: Option<i64>) -> TestArgs {
        TestArgs {
            id,
            enabled_only: false,
            active_only: false,
            selected_only: false,
            subscription: None,
            skip_icmp: false,
            skip_tcp: false,
            skip_real_delay: false,
            skip_download: false,
            test_url: None,
            download_url: None,
            icmp_timeout_ms: None,
            tcp_timeout_ms: None,
            real_delay_timeout_ms: None,
            download_timeout_ms: None,
            concurrency: None,
            format: crate::cli::TestFormat::Tsv,
            output: None,
            sort_by: crate::cli::TestSortBy::Status,
            no_progress: false,
        }
    }
}
