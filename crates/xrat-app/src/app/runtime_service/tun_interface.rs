use super::*;

impl RuntimeService<'_> {
    pub(crate) async fn begin_tun_ownership(&self, session_id: i64) -> crate::app::Result<()> {
        let record = tun_ownership::TunOwnershipRecord {
            interface_name: self.context.app_config.runtime.tun.interface_name.clone(),
            ifindex: None,
            interface_identity: None,
            session_id,
            engine: self.context.app_config.runtime.engine.clone(),
            policy_rules: vec![],
            kill_switch_active: false,
            kill_switch_backend: None,
        };
        if let Err(error) =
            tun_ownership::save_ownership(&self.context.runtime_paths.runtime_dir, &record)
        {
            self.context
                .db
                .update_runtime_session_state(
                    session_id,
                    RuntimeSessionStatus::Failed,
                    None,
                    None,
                    Some(&now_string()),
                    Some(&error.to_string()),
                )
                .await?;
            return Err(error.into());
        }
        Ok(())
    }

    pub(crate) async fn cleanup_inactive_tun_ownership(&self) -> crate::app::Result<()> {
        self.context.require_local_runtime()?;
        let Some(record) = tun_ownership::load_ownership(&self.context.runtime_paths.runtime_dir)
        else {
            return Ok(());
        };
        let Some(session) = self
            .context
            .db
            .get_latest_runtime_session()
            .await?
            .filter(|session| session.id == record.session_id)
        else {
            return Err(AppError::InvalidArgument(
                "TUN ownership has no matching latest session; refusing automatic cleanup".into(),
            ));
        };
        if matches!(
            session.status,
            RuntimeSessionStatus::Stopped | RuntimeSessionStatus::Failed
        ) && !runtime_session_is_alive(&session, self.process_ports.signals.as_ref())
        {
            self.cleanup_session_tun(session.id)?;
        }
        Ok(())
    }
    pub(crate) async fn cleanup_failed_tun_process(
        &self,
        session_id: i64,
        pid: u32,
        error: AppError,
    ) -> crate::app::Result<()> {
        let termination = xray_runtime::terminate_process_gracefully_with_signals(
            i64::from(pid),
            SHUTDOWN_TIMEOUT,
            self.process_ports.signals.as_ref(),
        );
        let cleanup = if termination.is_ok() {
            self.cleanup_session_tun(session_id)
        } else {
            Err(AppError::InvalidArgument(
                "TUN process termination failed; ownership retained".into(),
            ))
        };
        let combined_error = match cleanup {
            Ok(()) => error,
            Err(cleanup) => AppError::InvalidArgument(format!("{error}; {cleanup}")),
        };
        self.context
            .db
            .update_runtime_session_state(
                session_id,
                RuntimeSessionStatus::Failed,
                Some(i64::from(pid)),
                None,
                Some(&now_string()),
                Some(&combined_error.to_string()),
            )
            .await?;
        Err(combined_error)
    }

    pub(crate) async fn finish_tun_startup(
        &self,
        session_id: i64,
        pid: u32,
        allowed_endpoints: &[std::net::SocketAddr],
    ) -> crate::app::Result<()> {
        if !self.context.app_config.runtime.tun.enabled {
            return Ok(());
        }
        if let Err(error) = self.record_ready_tun_ownership(session_id, pid).await {
            return self
                .cleanup_failed_tun_process(session_id, pid, error)
                .await;
        }
        if self.context.app_config.runtime.tun.kill_switch.is_enabled()
            && let Err(error) = self.engage_killswitch(session_id, allowed_endpoints).await
        {
            return self
                .cleanup_failed_tun_process(session_id, pid, error)
                .await;
        }
        Ok(())
    }
    pub(crate) fn preflight_tun_policy_rules(&self) -> crate::app::Result<()> {
        let tun = &self.context.app_config.runtime.tun;
        #[cfg(windows)]
        if tun.enabled {
            crate::app::services::windows_tun::preflight(self.context)?;
        }
        if !tun.enabled || self.context.app_config.runtime.engine != "sing-box" {
            return Ok(());
        }
        let owned = tun_ownership::load_ownership(&self.context.runtime_paths.runtime_dir);
        #[cfg(target_os = "macos")]
        xrat_support::net::preflight_macos_capture(
            &tun.address,
            owned.as_ref().and_then(|record| {
                record
                    .ifindex
                    .map(|index| (record.interface_name.as_str(), index))
            }),
        )?;
        if !tun.auto_route {
            return Ok(());
        }
        for rule in self
            .process_ports
            .tun
            .policy_rules()?
            .into_iter()
            .filter(|rule| rule.is_singbox_capture_rule())
        {
            if !owned.as_ref().is_some_and(|record| {
                record.ifindex.is_some_and(|index| index > 0) && record.policy_rules.contains(&rule)
            }) {
                return Err(AppError::InvalidArgument("sing-box capture policy priorities 9000..9010 are already in use by an unowned rule; current connection preserved".into()));
            }
        }
        Ok(())
    }

    pub(crate) async fn record_ready_tun_ownership(
        &self,
        session_id: i64,
        pid: u32,
    ) -> crate::app::Result<()> {
        let tun = &self.context.app_config.runtime.tun;
        let singbox = self.context.app_config.runtime.engine == "sing-box";
        let deadline =
            tokio::time::Instant::now() + Duration::from_secs(if cfg!(windows) { 20 } else { 5 });
        loop {
            if let Some(info) = self
                .process_ports
                .tun
                .inspect_interface(&tun.interface_name)?
            {
                if !info.is_tun || info.ifindex == 0 {
                    return Err(AppError::InvalidArgument(
                        "new managed interface is not a verified kernel TUN".into(),
                    ));
                }
                let rules = if singbox && tun.auto_route {
                    self.process_ports
                        .tun
                        .policy_rules()?
                        .into_iter()
                        .filter(|rule| rule.is_singbox_capture_rule())
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                };
                tun_ownership::save_ownership(
                    &self.context.runtime_paths.runtime_dir,
                    &tun_ownership::TunOwnershipRecord {
                        interface_name: tun.interface_name.clone(),
                        ifindex: Some(info.ifindex),
                        interface_identity: self
                            .process_ports
                            .tun
                            .interface_identity(&tun.interface_name)?,
                        session_id,
                        engine: self.context.app_config.runtime.engine.clone(),
                        policy_rules: rules.clone(),
                        kill_switch_active: false,
                        kill_switch_backend: None,
                    },
                )?;
                #[cfg(windows)]
                {
                    let identity = self
                        .process_ports
                        .tun
                        .interface_identity(&tun.interface_name)?
                        .ok_or_else(|| {
                            AppError::InvalidArgument(
                                "Windows TUN has no verified platform identity".into(),
                            )
                        })?;
                    if !crate::app::services::windows_tun::ready(self.context, &identity)? {
                        if tokio::time::Instant::now() < deadline
                            && self.process_ports.signals.is_running(i64::from(pid))
                        {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            continue;
                        }
                        return Err(AppError::InvalidArgument("Windows TUN startup timeout: interface addresses, capture routes or DNS were not ready".into()));
                    }
                }
                #[cfg(target_os = "macos")]
                if singbox
                    && tun.auto_route
                    && !xrat_support::net::macos_capture_ready(&tun.address, &tun.interface_name)?
                {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                    if tokio::time::Instant::now() < deadline
                        && self.process_ports.signals.is_running(i64::from(pid))
                    {
                        continue;
                    }
                    return Err(AppError::InvalidArgument(
                        "macOS TUN startup timeout: capture routes were not ready".into(),
                    ));
                }
                if !singbox
                    || !tun.auto_route
                    || cfg!(any(target_os = "macos", windows))
                    || tun.address.iter().all(|address| {
                        let family = if address.contains(':') { 10 } else { 2 };
                        rules
                            .iter()
                            .any(|rule| rule.family == family && rule.priority() == Some(9010))
                    })
                {
                    return Ok(());
                }
            }
            if tokio::time::Instant::now() >= deadline
                || !self.process_ports.signals.is_running(i64::from(pid))
            {
                return Err(AppError::InvalidArgument(
                    "TUN startup timeout: verified interface and capture rules were not ready"
                        .into(),
                ));
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    pub(crate) fn cleanup_session_tun(&self, session_id: i64) -> crate::app::Result<()> {
        let Some(record) = tun_ownership::load_ownership(&self.context.runtime_paths.runtime_dir)
            .filter(|record| record.session_id == session_id)
        else {
            return Ok(());
        };
        let mut context = self.context.clone();
        context.app_config.runtime.tun.interface_name = record.interface_name;
        RuntimeService::with_process_ports(&context, self.process_ports.clone())
            .cleanup_stale_tun_interface()
    }
    pub(crate) fn verify_tun_interface(
        &self,
    ) -> crate::app::Result<Option<xrat_support::net::KernelInterfaceInfo>> {
        let interface = self.context.app_config.runtime.tun.interface_name.trim();
        if interface.is_empty() {
            return Ok(None);
        }
        let Some(info) = self
            .process_ports
            .tun
            .inspect_interface(interface)
            .map_err(|error| {
                AppError::InvalidArgument(format!(
                    "failed to inspect network interface \"{interface}\": {error}"
                ))
            })?
        else {
            return Ok(None);
        };

        if !info.is_tun {
            return Err(AppError::InvalidArgument(format!(
                "network interface \"{interface}\" already exists and is not a TUN device; refusing to remove non-TUN interface"
            )));
        }

        let Some(record) = tun_ownership::load_ownership(&self.context.runtime_paths.runtime_dir)
        else {
            let guidance = if cfg!(target_os = "macos") {
                "Stop the application that owns this utun interface before retrying.".to_string()
            } else if cfg!(windows) {
                "Stop the application that owns this Windows adapter before retrying.".to_string()
            } else {
                format!("Remove it manually, for example with `sudo ip link del {interface}`.")
            };
            return Err(AppError::InvalidArgument(format!(
                "TUN interface \"{interface}\" already exists but its ownership by XRAT could not be verified; refusing to remove unowned interface. {guidance}"
            )));
        };

        if record.interface_name != interface {
            return Err(AppError::InvalidArgument(format!(
                "TUN interface \"{interface}\" already exists but does not match owned interface \"{}\"; refusing to remove unowned interface.",
                record.interface_name
            )));
        }

        let Some(expected_ifindex) = record.ifindex.filter(|index| *index > 0) else {
            return Err(AppError::InvalidArgument(format!(
                "TUN interface \"{interface}\" has no verified kernel index; refusing to remove unverified interface"
            )));
        };
        if info.ifindex != expected_ifindex {
            return Err(AppError::InvalidArgument(format!(
                "TUN interface \"{interface}\" (index {}) does not match previously recorded interface index {expected_ifindex}; refusing to remove unverified interface.",
                info.ifindex
            )));
        }
        if self.process_ports.tun.interface_identity(interface)? != record.interface_identity {
            return Err(AppError::InvalidArgument(format!(
                "TUN interface \"{interface}\" no longer matches its recorded platform identity; refusing automatic cleanup"
            )));
        }
        Ok(Some(info))
    }

    pub(crate) fn cleanup_stale_tun_interface(&self) -> crate::app::Result<()> {
        let record = tun_ownership::load_ownership(&self.context.runtime_paths.runtime_dir);
        let info = self.verify_tun_interface()?;
        if let Some(record) = &record
            && !record.policy_rules.is_empty()
        {
            if record.engine != "sing-box" || record.ifindex.is_none_or(|index| index == 0) {
                return Err(AppError::InvalidArgument(
                    "policy cleanup requires verified sing-box TUN ownership".into(),
                ));
            }
            let current = self.process_ports.tun.policy_rules()?;
            for rule in &record.policy_rules {
                if current.contains(rule) {
                    self.process_ports.tun.delete_policy_rule(rule)?;
                }
            }
        }
        let Some(info) = info else {
            #[cfg(windows)]
            if let Some(identity) = record
                .as_ref()
                .and_then(|record| record.interface_identity.as_deref())
            {
                crate::app::services::windows_tun::verify_release(identity)?;
            }
            tun_ownership::clear_ownership(&self.context.runtime_paths.runtime_dir);
            return Ok(());
        };
        let interface = self.context.app_config.runtime.tun.interface_name.trim();
        self.process_ports
            .tun
            .delete_interface(interface, info.ifindex)
            .map_err(|error| {
                let guidance = if cfg!(target_os = "macos") {
                    "Ensure the engine has exited and released its utun socket.".to_string()
                } else if cfg!(windows) {
                    "Ensure the engine has exited and released its Wintun adapter; the ownership journal is retained for retry.".to_string()
                } else {
                    format!("Ensure CAP_NET_ADMIN or remove it manually, for example with `sudo ip link del {interface}`.")
                };
                AppError::InvalidArgument(format!(
                    "stale TUN interface \"{interface}\" could not be removed ({error}). {guidance}"
                ))
            })?;

        #[cfg(windows)]
        if let Some(identity) = record
            .as_ref()
            .and_then(|record| record.interface_identity.as_deref())
        {
            crate::app::services::windows_tun::verify_release(identity)?;
        }
        tracing::info!(
            interface,
            ifindex = info.ifindex,
            "removed stale TUN interface before launch"
        );
        tun_ownership::clear_ownership(&self.context.runtime_paths.runtime_dir);
        Ok(())
    }
}
