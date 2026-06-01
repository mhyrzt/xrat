use super::TestArgs;
use crate::db::ConfigListFilter;

impl TestArgs {
    pub fn config_filter(&self) -> ConfigListFilter {
        ConfigListFilter {
            only_enabled: self.enabled_only,
            only_selected: self.selected_only,
            only_active: self.active_only,
            only_deleted: false,
            include_deleted: false,
            subscription_id: self.subscription,
            protocol: None,
        }
    }
}
