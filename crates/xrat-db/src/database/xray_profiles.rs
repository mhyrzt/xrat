use super::Database;
use crate::{ImportSource, ImportSummary, XrayProfileRecord};
use xrat_model::ConfigId;

impl Database {
    pub async fn import_xray_profile(
        &self,
        source: &ImportSource,
        digest: &str,
        byte_length: i64,
        name: Option<&str>,
    ) -> crate::Result<ImportSummary> {
        crate::repository::xray_profiles::import(&self.pool, source, digest, byte_length, name)
            .await
    }

    pub async fn get_xray_profile(&self, id: ConfigId) -> crate::Result<Option<XrayProfileRecord>> {
        crate::repository::xray_profiles::get(&self.pool, id).await
    }

    pub async fn xray_profile_digests(&self) -> crate::Result<Vec<String>> {
        crate::repository::xray_profiles::digests(&self.pool).await
    }
}
