use super::*;

pub(crate) async fn run_download_stage(
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
        &settings.gen_options,
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

pub(crate) async fn run_upload_stage(
    node: &Node,
    settings: &ResolvedTestSettings,
    result: &mut TestResult,
    print_progress: bool,
) -> crate::app::Result<()> {
    let Some(upload_url) = settings.upload_url.as_deref() else {
        return Ok(());
    };

    if print_progress {
        print!("Running upload speed test... ");
        std::io::stdout().flush()?;
    }

    let upload_result = upload_speed_check(
        node,
        upload_url,
        &settings.xray_binary_path,
        settings.xray_startup_timeout,
        settings.upload_timeout,
        settings.upload_payload_bytes,
        &settings.gen_options,
    )
    .await;
    let failure_reason = upload_result.failure_reason.clone();

    result.upload_ok = upload_result.success;
    result.upload_mbps = upload_result.mbps;
    merge_failure(
        result,
        upload_result.failure_kind,
        upload_result.failure_reason,
    );

    if print_progress {
        print_download_result(
            upload_result.success,
            upload_result.mbps,
            failure_reason.as_deref(),
        );
    }

    Ok(())
}
