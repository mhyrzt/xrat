use super::TestArgs;
use crate::db::ConfigListFilter;

impl TestArgs {
    pub fn config_filter(&self, subscription_id: Option<i64>) -> ConfigListFilter {
        ConfigListFilter {
            only_enabled: self.enabled_only,
            only_active: self.active_only,
            only_deleted: false,
            include_deleted: false,
            subscription_id,
            protocol: None,
        }
    }
}
