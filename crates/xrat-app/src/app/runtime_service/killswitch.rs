use super::*;
use xrat_support::net::KillSwitchConfig;

impl<'a> RuntimeService<'a> {
    pub(crate) async fn engage_killswitch(
        &self,
        session_id: i64,
        allowed_endpoints: &[std::net::SocketAddr],
    ) -> crate::app::Result<()> {
        let tun = &self.context.app_config.runtime.tun;
        if !tun.kill_switch.is_enabled() {
            return Ok(());
        }

        let socket_mark = if self.context.app_config.runtime.network.mark > 0 {
            Some(self.context.app_config.runtime.network.mark as u32)
        } else {
            Some(0x255)
        };

        let config = KillSwitchConfig {
            tun_interface: tun.interface_name.clone(),
            allowed_endpoints: allowed_endpoints.to_vec(),
            socket_mark,
            allow_lan: tun.allow_lan,
            block_ipv6: false,
        };

        self.process_ports
            .firewall
            .apply_killswitch(&config)
            .map_err(|err| {
                AppError::InvalidArgument(format!("Failed to activate TUN kill switch: {err}"))
            })?;

        #[allow(clippy::collapsible_if)]
        if let Some(mut record) =
            tun_ownership::load_ownership(&self.context.runtime_paths.runtime_dir)
        {
            if record.session_id == session_id {
                record.kill_switch_active = true;
                record.kill_switch_backend =
                    Some(self.process_ports.firewall.backend_name().to_string());
                let _ =
                    tun_ownership::save_ownership(&self.context.runtime_paths.runtime_dir, &record);
            }
        }

        crate::app::events::record(
            &self.context.db,
            crate::app::events::LEVEL_INFO,
            crate::app::events::SOURCE_RUNTIME,
            "tun.killswitch.engaged",
            "TUN fail-closed kill switch successfully activated",
            None,
            Some(session_id),
            None,
        )
        .await;

        Ok(())
    }

    pub(crate) async fn disengage_killswitch(&self) -> crate::app::Result<()> {
        let is_active = self.process_ports.firewall.is_active().map_err(|error| {
            AppError::InvalidArgument(format!("Failed to query TUN kill switch: {error}"))
        })?;
        let owned_active = tun_ownership::load_ownership(&self.context.runtime_paths.runtime_dir)
            .is_some_and(|r| r.kill_switch_active);
        if !is_active
            && !owned_active
            && !self.context.app_config.runtime.tun.kill_switch.is_enabled()
        {
            return Ok(());
        }

        self.process_ports
            .firewall
            .remove_killswitch()
            .map_err(|err| {
                AppError::InvalidArgument(format!("Failed to deactivate TUN kill switch: {err}"))
            })?;

        if let Some(mut record) =
            tun_ownership::load_ownership(&self.context.runtime_paths.runtime_dir)
        {
            record.kill_switch_active = false;
            let _ = tun_ownership::save_ownership(&self.context.runtime_paths.runtime_dir, &record);
        }

        crate::app::events::record(
            &self.context.db,
            crate::app::events::LEVEL_INFO,
            crate::app::events::SOURCE_RUNTIME,
            "tun.killswitch.disengaged",
            "TUN kill switch deactivated",
            None,
            None,
            None,
        )
        .await;

        Ok(())
    }
}
