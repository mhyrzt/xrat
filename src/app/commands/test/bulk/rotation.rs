use super::super::*;

pub(crate) async fn run_rotation_bulk_tests(
    context: &AppContext,
    candidate_ids: &[i64],
) -> crate::app::Result<Vec<TestOutputRow>> {
    if candidate_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut settings = resolve_test_settings(
        &TestArgs {
            id: None,
            test_url: None,
            download_url: None,
            upload_url: None,
            upload_timeout_ms: None,
            skip_icmp: false,
            skip_real_delay: false,
            skip_download: false,
            skip_upload: true,
            skip_tcp: false,
            sort_by: TestSortBy::Status,
            format: TestFormat::Tsv,
            output: None,
            country: None,
            asn: None,
            enabled_only: false,
            active_only: false,
            subscription: None,
            latest_run_summary: false,
            ping: false,
            ping_interval_ms: 1000,
            icmp_timeout_ms: None,
            real_delay_timeout_ms: None,
            download_timeout_ms: None,
            tcp_timeout_ms: None,
            concurrency: None,
            no_progress: true,
        },
        &context.app_config,
        &context.runtime_paths,
    )?;

    settings.concurrency = context.app_config.runtime.rotation.test_concurrency;
    settings.stage_order = context
        .app_config
        .runtime
        .rotation
        .test_stages
        .iter()
        .filter_map(|stage| ConnectionTestStage::from_config_str(stage))
        .collect();
    settings.run_icmp = context
        .app_config
        .runtime
        .rotation
        .test_stages
        .iter()
        .any(|stage| stage == "icmp")
        && context.app_config.testing.icmp.enabled;
    settings.run_upload = false;

    let has_real_delay = context
        .app_config
        .runtime
        .rotation
        .test_stages
        .iter()
        .any(|stage| stage == "real_delay");
    let has_download = context
        .app_config
        .runtime
        .rotation
        .test_stages
        .iter()
        .any(|stage| stage == "download");
    let has_tcp = context
        .app_config
        .runtime
        .rotation
        .test_stages
        .iter()
        .any(|stage| stage == "tcp");
    settings.run_real_delay = has_real_delay && context.app_config.testing.real_delay.enabled;
    settings.run_download = has_download && context.app_config.testing.download.enabled;
    settings.run_tcp =
        (has_tcp || settings.run_real_delay) && context.app_config.testing.tcp.enabled;

    if !settings.run_real_delay && !settings.run_download && !settings.run_tcp {
        return Ok(Vec::new());
    }

    let configs = context
        .db
        .list_configs(&crate::db::ConfigListFilter {
            only_enabled: true,
            ..Default::default()
        })
        .await?
        .into_iter()
        .filter(|config| candidate_ids.contains(&config.id))
        .collect::<Vec<_>>();

    run_bulk_for_configs(context, settings, configs, "rotation", false).await
}
