use super::super::Database;
use super::super::types::*;

impl Database {
    pub async fn delete_config(&self, id: i64) -> crate::db::Result<()> {
        repository::delete_config(&self.pool, id).await
    }

    pub async fn restore_config(&self, id: i64) -> crate::db::Result<()> {
        repository::restore_config(&self.pool, id).await
    }

    pub async fn hard_delete_config(&self, id: i64) -> crate::db::Result<()> {
        repository::hard_delete_config(&self.pool, id).await
    }

    pub async fn set_selected_config(&self, id: i64) -> crate::db::Result<()> {
        repository::set_selected_config(&self.pool, id).await
    }

    pub async fn set_active_config(&self, id: i64) -> crate::db::Result<()> {
        repository::set_active_config(&self.pool, id).await
    }

    pub async fn clear_active_config(&self) -> crate::db::Result<()> {
        repository::clear_active_config(&self.pool).await
    }

    pub async fn set_config_enabled(&self, id: i64, is_enabled: bool) -> crate::db::Result<()> {
        repository::set_config_enabled(&self.pool, id, is_enabled).await
    }
}
