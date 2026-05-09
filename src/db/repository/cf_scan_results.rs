use sqlx::QueryBuilder;

use super::row::map_cf_scan_result_row;
use crate::db::connection::DbPool;
use crate::db::model::{CfScanResultRecord, CfScanResultUpsert};

pub async fn upsert_batch(pool: &DbPool, results: &[CfScanResultUpsert]) -> crate::db::Result<()> {
    if results.is_empty() {
        return Ok(());
    }

    match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::new(
                "INSERT INTO cf_scan_results (ip, latency_ms, download_mbps, upload_mbps, error, last_scanned_at) ",
            );
            builder.push_values(results, |mut row, result| {
                row.push_bind(&result.ip)
                    .push_bind(result.latency_ms)
                    .push_bind(result.download_mbps)
                    .push_bind(result.upload_mbps)
                    .push_bind(&result.error)
                    .push("CURRENT_TIMESTAMP");
            });
            builder.push(
                " ON CONFLICT(ip) DO UPDATE SET \
                latency_ms = COALESCE(excluded.latency_ms, cf_scan_results.latency_ms), \
                download_mbps = COALESCE(excluded.download_mbps, cf_scan_results.download_mbps), \
                upload_mbps = COALESCE(excluded.upload_mbps, cf_scan_results.upload_mbps), \
                error = excluded.error, \
                last_scanned_at = CURRENT_TIMESTAMP",
            );
            builder.build().execute(pool).await?;
            Ok(())
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::new(
                "INSERT INTO cf_scan_results (ip, latency_ms, download_mbps, upload_mbps, error, last_scanned_at) ",
            );
            builder.push_values(results, |mut row, result| {
                row.push_bind(&result.ip)
                    .push_bind(result.latency_ms)
                    .push_bind(result.download_mbps)
                    .push_bind(result.upload_mbps)
                    .push_bind(&result.error)
                    .push("CURRENT_TIMESTAMP");
            });
            builder.push(
                " ON CONFLICT(ip) DO UPDATE SET \
                latency_ms = COALESCE(EXCLUDED.latency_ms, cf_scan_results.latency_ms), \
                download_mbps = COALESCE(EXCLUDED.download_mbps, cf_scan_results.download_mbps), \
                upload_mbps = COALESCE(EXCLUDED.upload_mbps, cf_scan_results.upload_mbps), \
                error = EXCLUDED.error, \
                last_scanned_at = CURRENT_TIMESTAMP",
            );
            builder.build().execute(pool).await?;
            Ok(())
        }
    }
}

pub async fn list_all(pool: &DbPool) -> crate::db::Result<Vec<CfScanResultRecord>> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query(
            "SELECT id, ip, latency_ms, download_mbps, upload_mbps, error, last_scanned_at FROM cf_scan_results ORDER BY id ASC",
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(map_cf_scan_result_row)
        .collect()),
        DbPool::Postgres(pool) => Ok(sqlx::query(
            "SELECT id, ip, latency_ms, download_mbps, upload_mbps, error, last_scanned_at FROM cf_scan_results ORDER BY id ASC",
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(map_cf_scan_result_row)
        .collect()),
    }
}

pub async fn list_history(pool: &DbPool, limit: i64) -> crate::db::Result<Vec<CfScanResultRecord>> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query(
            "SELECT id, ip, latency_ms, download_mbps, upload_mbps, error, last_scanned_at
             FROM cf_scan_results
             ORDER BY
                CASE WHEN error IS NULL THEN 0 ELSE 1 END,
                latency_ms ASC,
                download_mbps DESC
             LIMIT ?1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(map_cf_scan_result_row)
        .collect()),
        DbPool::Postgres(pool) => Ok(sqlx::query(
            "SELECT id, ip, latency_ms, download_mbps, upload_mbps, error, last_scanned_at
             FROM cf_scan_results
             ORDER BY
                CASE WHEN error IS NULL THEN 0 ELSE 1 END,
                latency_ms ASC,
                download_mbps DESC
             LIMIT $1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(map_cf_scan_result_row)
        .collect()),
    }
}
