use crate::app::{
    self,
    services::{runtime_control, testing},
};
use xrat_model::{ConfigId, ConfigRef};
use xrat_web::types::Action;
mod test_progress;
pub(crate) async fn execute(
    context: &app::context::AppContext,
    action: Action,
    flag: &xrat_support::cancel::CancellationFlag,
    job_id: &str,
) -> app::Result<String> {
    let mut context = context.clone();
    if !context.is_machine_client() {
        super::tun_control::reload(&mut context)?;
    }
    let context = &context;
    let services = context.services();
    match action {
        Action::Bulk { operation, ids } => {
            if ids.is_empty()
                || ids.len() > 500
                || ids.iter().any(|id| *id <= 0)
                || ids.iter().collect::<std::collections::HashSet<_>>().len() != ids.len()
            {
                return Err(app::AppError::InvalidArgument(
                    "Select 1–500 unique rows.".into(),
                ));
            }
            if operation == xrat_web::types::BulkOperation::Test {
                let ids = ids.into_iter().map(ConfigId).collect::<Vec<_>>();
                return test_progress::run(context, &ids, flag, job_id).await;
            }
            let total = ids.len();
            let mut completed = 0;
            let mut failed = Vec::new();
            for id in ids {
                if flag.is_cancelled() {
                    break;
                }
                match Box::pin(execute(context, operation.action(id), flag, job_id)).await {
                    Ok(_) => completed += 1,
                    Err(error) => {
                        tracing::warn!(%error, id, "bulk item failed");
                        failed.push(id);
                    }
                }
                if let Err(error) = context
                    .db
                    .web_job_progress(
                        job_id,
                        &format!(
                            "{}: {completed} completed, {} failed of {total}",
                            operation.label(),
                            failed.len()
                        ),
                        timestamp(),
                    )
                    .await
                {
                    tracing::warn!(%error, "could not record bulk progress");
                }
            }
            if !failed.is_empty() {
                return Err(app::AppError::InvalidArgument(format!(
                    "{completed} completed; failed IDs: {failed:?}"
                )));
            }
            return Ok(format!(
                "{}: {completed} of {total} completed",
                operation.label()
            ));
        }
        Action::Add { input } => {
            super::catalog_import::add(context, &input).await?;
        }
        Action::Import { input, name } => {
            if super::xray_profiles::parse(input.as_bytes())?.is_some() {
                return Err(app::AppError::InvalidArgument("Complete Xray profiles require local CLI/TUI import; remote uploads cannot provision arbitrary engine configuration".into()));
            }
            if !input.starts_with("http://") && !input.starts_with("https://") {
                let nodes = app::import::parse_nodes(input.as_bytes())?;
                if nodes.is_empty() {
                    return Err(app::AppError::NoSupportedConfig);
                }
                let source = xrat_db::ImportSource {
                    kind: xrat_db::SourceKind::RawText,
                    value: "web upload".into(),
                    name: name.clone(),
                };
                super::catalog_import::persist_nodes(
                    context,
                    source,
                    &nodes,
                    name.as_deref(),
                    None,
                )
                .await?;
            } else {
                super::catalog_import::subscription(
                    context,
                    &input,
                    name.as_deref(),
                    &Default::default(),
                )
                .await?;
            }
        }
        Action::Enable { id } => {
            services
                .lifecycle
                .enable(&ConfigRef::from(id.to_string()))
                .await?;
        }
        Action::Disable { id } => {
            services
                .lifecycle
                .disable(&ConfigRef::from(id.to_string()))
                .await?;
        }
        Action::Delete { id, hard } => {
            services
                .lifecycle
                .delete(&ConfigRef::from(id.to_string()), hard)
                .await?;
        }
        Action::Restore { id } => {
            services
                .lifecycle
                .restore(&ConfigRef::from(id.to_string()))
                .await?;
        }
        Action::DeleteSubscription { id } => {
            services
                .lifecycle
                .delete_subscription(&id.to_string())
                .await?;
        }
        Action::RenameSubscription { id, name } => {
            services
                .lifecycle
                .rename_subscription(&id.to_string(), &name)
                .await?;
        }
        Action::Connect { id } => {
            runtime_control::tui_control(context)
                .await?
                .connect(ConfigId(id))
                .await?;
        }
        Action::Disconnect => {
            runtime_control::tui_control(context)
                .await?
                .disconnect()
                .await?;
        }
        Action::Rotate => {
            runtime_control::tui_control(context)
                .await?
                .replace(app::daemon::ipc::RotationTrigger::Manual, None)
                .await?;
        }
        Action::Refresh { id } => {
            let subscriptions = services.configs.subscriptions().await?;
            for source in subscriptions
                .into_iter()
                .filter(|source| id.is_none_or(|id| source.id.0 == id))
            {
                if let Some(url) = source.source_url {
                    super::catalog_import::subscription(context, &url, None, &source.headers)
                        .await?;
                }
            }
        }
        Action::Upgrade { version } => {
            if context.is_machine_client() {
                return Err(app::AppError::InvalidArgument(
                    "Machine upgrades require local administrator maintenance".into(),
                ));
            }
            let version = match version {
                Some(version) => version,
                None => {
                    super::releases::ReleaseService::default()
                        .latest_tag(120)
                        .await?
                }
            };
            let candidate = semver::Version::parse(version.trim_start_matches('v'))
                .map_err(|_| app::AppError::InvalidArgument("invalid release version".into()))?;
            let current = semver::Version::parse(env!("CARGO_PKG_VERSION"))
                .map_err(|_| app::AppError::InvalidArgument("invalid current version".into()))?;
            if candidate < current {
                return Err(app::AppError::InvalidArgument(
                    "browser upgrades cannot downgrade XRAT".into(),
                ));
            }
            let outcome = super::upgrades::run(
                &context.runtime_paths.config_path,
                &super::upgrades::UpgradeOptions {
                    version: Some(version),
                    timeout_secs: 120,
                    no_tun_setup: true,
                    no_restart_daemon: true,
                    ..Default::default()
                },
            )
            .await?;
            return Ok(outcome.message());
        }
        Action::Rotation { enabled } => {
            #[cfg(any(windows, test))]
            if let Some(client) = &context.machine_client {
                use super::machine_service::GatewayRequestKind;
                client
                    .call(if enabled {
                        GatewayRequestKind::RotationStart {}
                    } else {
                        GatewayRequestKind::RotationStop {}
                    })
                    .await?;
                return Ok(if enabled {
                    "Rotation enabled"
                } else {
                    "Rotation disabled"
                }
                .into());
            }
            let socket = app::daemon::ipc::default_socket_path(&context.runtime_paths.runtime_dir);
            let response = if enabled {
                app::daemon::ipc::proxy_start_daemon(&socket).await?
            } else {
                app::daemon::ipc::proxy_stop_daemon(&socket).await?
            };
            if !response.ok {
                return Err(app::AppError::InvalidArgument(response.message));
            }
            let mut editor =
                app::config::ConfigEditSession::open(&context.runtime_paths.config_path)
                    .map_err(app::AppError::InvalidArgument)?;
            let setting = editor
                .settings
                .iter_mut()
                .find(|s| s.path == "runtime.rotation.enabled")
                .ok_or_else(|| {
                    app::AppError::InvalidArgument("rotation setting not found".into())
                })?;
            setting.value = app::config::SettingValue::Bool(enabled);
            editor.save().map_err(app::AppError::InvalidArgument)?;
            return Ok(if enabled {
                "Rotation enabled"
            } else {
                "Rotation disabled"
            }
            .into());
        }
        Action::GeoIpLookup { address } => {
            let ip = xrat_support::geoip::resolve_address_ip(&address)
                .await
                .ok_or_else(|| {
                    app::AppError::InvalidArgument("address could not be resolved".into())
                })?;
            let lookup = app::geoip_backend::build_lookup_chain(
                &context.app_config,
                &context.runtime_paths,
            )?;
            return Ok(format!(
                "{ip}: {} / {} / {}",
                lookup.country(ip).await?.unwrap_or_default(),
                lookup.city(ip).await?.unwrap_or_default(),
                lookup.asn(ip).await?.unwrap_or_default()
            ));
        }
        Action::GeoIpUpdate => {
            if context.is_machine_client() {
                return Err(app::AppError::InvalidArgument(
                    "Machine GeoIP updates require local administrator maintenance".into(),
                ));
            }
            return super::geoip_assets::download_assets(context).await;
        }
        Action::Validate => {
            if context.is_machine_client() {
                return Err(app::AppError::InvalidArgument(
                    "Machine policy validation requires local administrator access".into(),
                ));
            }
            let config = app::config::load(&context.runtime_paths.config_path)?;
            let errors = app::commands::validate::validate_app_config(&config);
            return Ok(if errors.is_empty() {
                "Configuration is valid".into()
            } else {
                errors.join("\n")
            });
        }
        Action::Parse { input } => {
            let nodes = xrat_config::parse_text(&input);
            return Ok(format!(
                "Parsed {} configs: {}",
                nodes.len(),
                nodes
                    .iter()
                    .map(|node| node.protocol.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        Action::Purge => {
            return Ok(format!(
                "Purged {} configs",
                services.lifecycle.purge().await?
            ));
        }
        Action::ClearEvents => {
            super::diagnostics::DiagnosticsService::new(context.clone())
                .dispatch(super::diagnostics::DiagnosticsRequest::ClearEvents {})
                .await?;
        }
        Action::Tun { enabled } => {
            let mut context = context.clone();
            let result = super::tun_control::apply(&mut context, Some(enabled), true).await?;
            return Ok(super::tun_control::message(&result));
        }
        Action::InstallEngine { engine } => {
            if context.is_machine_client() {
                return Err(app::AppError::InvalidArgument("Machine core installation requires a local administrator command: xrat --profile machine install <core>".into()));
            }
            let kind = match engine {
                xrat_web::types::Engine::Xray => super::cores::CoreKind::Xray,
                xrat_web::types::Engine::SingBox => super::cores::CoreKind::SingBox,
                xrat_web::types::Engine::V2ray => super::cores::CoreKind::V2Ray,
            };
            let client = xrat_support::http::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()?;
            let release = super::cores::fetch_release(&client, kind, None, false)
                .await
                .map_err(app::AppError::InvalidArgument)?;
            let result = super::cores::install(context, kind, &release, false)
                .await
                .map_err(app::AppError::InvalidArgument)?;
            return Ok(format!("Installed {} {}", kind.name(), result.version));
        }
        Action::Scan {
            ips,
            port,
            timeout_ms,
        } => {
            if ips.is_empty()
                || ips.len() > 4096
                || port == 0
                || !(100..=60000).contains(&timeout_ms)
            {
                return Err(app::AppError::InvalidArgument(
                    "scan requires 1-4096 IPs, a nonzero port, and timeout 100-60000 ms".into(),
                ));
            }
            let ips = ips
                .iter()
                .map(|ip| ip.parse::<std::net::IpAddr>().map(|ip| ip.to_string()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| {
                    app::AppError::InvalidArgument("scanner accepts IP addresses only".into())
                })?;
            let mut rows = Vec::new();
            let total = ips.len();
            for ip in ips {
                if flag.is_cancelled() {
                    break;
                }
                let result =
                    xrat_prober::tcp_check(&ip, port, std::time::Duration::from_millis(timeout_ms))
                        .await;
                rows.push(xrat_db::CfScanResultUpsert {
                    ip,
                    latency_ms: result.latency_ms.map(i64::from),
                    download_mbps: None,
                    upload_mbps: None,
                    error: result.failure_reason,
                });
                if let Err(error) = context
                    .db
                    .web_job_progress(
                        job_id,
                        &format!("Scanned {} of {total} addresses", rows.len()),
                        timestamp(),
                    )
                    .await
                {
                    tracing::warn!(%error,"could not record scan progress");
                }
            }
            context
                .db
                .web_job_progress(
                    job_id,
                    &format!("Scanned {} of {total} addresses", rows.len()),
                    timestamp(),
                )
                .await?;
            context.db.upsert_cf_scan_results(&rows).await?;
            return Ok(format!("Scanned {} IP addresses", rows.len()));
        }
        Action::Test { id } => {
            let ids = services
                .configs
                .list(&super::ConfigListRequest {
                    only_enabled: id.is_none(),
                    ..Default::default()
                })
                .await?
                .items
                .into_iter()
                .filter(|row| id.is_none_or(|id| row.summary.id.0 == id))
                .map(|row| row.summary.id)
                .collect::<Vec<_>>();
            return test_progress::run(context, &ids, flag, job_id).await;
        }
    }
    Ok("Completed".into())
}

fn timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
#[path = "web_management_tests.rs"]
mod tests;
