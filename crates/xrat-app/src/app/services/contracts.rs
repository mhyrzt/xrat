use super::*;

/// Application services shared by every interface (CLI, TUI, HTTP, daemon).
///
/// Construct once per process and pass to handlers. Services depend on ports,
/// not on concrete infrastructure, so tests can build an `AppServices` with
/// fakes.
#[derive(Clone)]
pub struct AppServices {
    pub configs: ConfigService,
    pub lifecycle: ConfigLifecycleService,
    pub clock: Arc<dyn Clock>,
    pub filesystem: Arc<dyn Filesystem>,
}

impl AppServices {
    /// Wire production services from an application context.
    pub fn from_context(context: &AppContext) -> Self {
        #[cfg(any(windows, test))]
        if let Some(client) = &context.machine_client {
            return Self::from_machine(client.clone());
        }
        let repository = Arc::new(
            DatabaseConfigRepository::new(context.db.clone())
                .with_profiles(super::xray_profiles::ProfileStore::for_context(context)),
        );
        Self {
            configs: ConfigService::new(Arc::clone(&repository)),
            lifecycle: ConfigLifecycleService::new(repository),
            clock: Arc::new(SystemClock),
            filesystem: Arc::new(RealFilesystem),
        }
    }

    /// Wire catalog operations through the machine gateway, with no local persistence fallback.
    #[cfg(any(windows, test))]
    pub fn from_machine(client: super::machine_service::MachineClient) -> Self {
        Self {
            configs: ConfigService::for_machine(client.clone()),
            lifecycle: ConfigLifecycleService::for_machine(client),
            clock: Arc::new(SystemClock),
            filesystem: Arc::new(RealFilesystem),
        }
    }

    /// Wire production services from a database handle alone.
    ///
    /// Used by hosts that only need config services, such as the HTTP API
    /// server, without building a full [`AppContext`].
    pub fn from_database(db: xrat_db::Database) -> Self {
        let repository = Arc::new(DatabaseConfigRepository::new(db));
        Self {
            configs: ConfigService::new(Arc::clone(&repository)),
            lifecycle: ConfigLifecycleService::new(repository),
            clock: Arc::new(SystemClock),
            filesystem: Arc::new(RealFilesystem),
        }
    }

    /// Wire services with explicit ports, for tests and alternative hosts.
    pub fn with_ports(
        configs: ConfigService,
        lifecycle: ConfigLifecycleService,
        clock: Arc<dyn Clock>,
        filesystem: Arc<dyn Filesystem>,
    ) -> Self {
        Self {
            configs,
            lifecycle,
            clock,
            filesystem,
        }
    }
}
