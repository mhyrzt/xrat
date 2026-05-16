#[cfg(test)]
pub(super) use std::path::PathBuf;
#[cfg(test)]
pub(super) use std::time::Duration;
#[cfg(test)]
pub(super) use std::time::{SystemTime, UNIX_EPOCH};

pub(super) use crate::db::connection::{self, DatabaseConnectionConfig, DbPool};
pub(super) use crate::db::record::{
    CfScanResultRecord, CfScanResultUpsert, ConfigListFilter, ConfigRecord, ConnectionTestInsert,
    ConnectionTestRecord, ConnectionTestRunInsert, ConnectionTestRunRecord, ImportSource,
    ImportSummary, RuntimeSessionInsert, RuntimeSessionRecord, RuntimeSessionStatus,
    SubscriptionRecord,
};
pub(super) use crate::db::repository;

#[derive(Clone)]
pub struct Database {
    pub(super) pool: DbPool,
}
