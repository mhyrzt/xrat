use crate::db::connection::DbPool;
use crate::db::model::{CfScanResultRecord, CfScanResultUpsert};
use crate::db::repository::cf_scan_results;

pub async fn upsert_cf_scan_results(
    pool: &DbPool,
    results: &[CfScanResultUpsert],
) -> crate::db::Result<()> {
    cf_scan_results::upsert_batch(pool, results).await
}

pub async fn list_cf_scan_results(pool: &DbPool) -> crate::db::Result<Vec<CfScanResultRecord>> {
    cf_scan_results::list_all(pool).await
}

pub async fn list_cf_scan_history(
    pool: &DbPool,
    limit: i64,
) -> crate::db::Result<Vec<CfScanResultRecord>> {
    cf_scan_results::list_history(pool, limit).await
}
