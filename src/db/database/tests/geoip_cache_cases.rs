use super::{Database, test_database_path};
use crate::db::GeoIpCacheUpsert;

fn entry(host: &str, country: Option<&str>, resolved_at: i64) -> GeoIpCacheUpsert {
    GeoIpCacheUpsert {
        host: host.to_string(),
        ip: None,
        country: country.map(str::to_string),
        location: country.map(|code| format!("{code}/City")),
        asn: country.map(|_| "AS65000 EXAMPLE".to_string()),
        resolved_at,
    }
}

/// Backend-agnostic checks reused by the SQLite test below and the PostgreSQL
/// verification path so both dialects exercise upsert / get_fresh / TTL.
pub(super) async fn verify_geoip_cache_state(db: &Database) {
    let now = 1_700_000_000_i64;

    db.upsert_geoip_cache(&entry("located.example", Some("US"), now))
        .await
        .expect("upsert located host");
    db.upsert_geoip_cache(&entry("dead.example", None, now))
        .await
        .expect("upsert empty host");

    let fresh = db
        .get_fresh_geoip_cache(
            &[
                "located.example".to_string(),
                "dead.example".to_string(),
                "never-seen.example".to_string(),
            ],
            now - 10,
        )
        .await
        .expect("fresh read should succeed");
    assert_eq!(fresh.len(), 2);
    let located = fresh
        .iter()
        .find(|record| record.host == "located.example")
        .expect("located host present");
    assert!(located.has_location());
    assert_eq!(located.country.as_deref(), Some("US"));
    let dead = fresh
        .iter()
        .find(|record| record.host == "dead.example")
        .expect("empty host cached as a recent attempt");
    assert!(!dead.has_location());

    // Entries older than the cutoff are treated as misses.
    let stale = db
        .get_fresh_geoip_cache(&["located.example".to_string()], now + 10)
        .await
        .expect("stale read should succeed");
    assert!(stale.is_empty());

    // Upsert overwrites geo and refreshes resolved_at for the same host.
    db.upsert_geoip_cache(&entry("located.example", Some("GB"), now + 100))
        .await
        .expect("overwrite located host");
    let updated = db
        .get_fresh_geoip_cache(&["located.example".to_string()], now)
        .await
        .expect("updated read should succeed");
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].country.as_deref(), Some("GB"));
}

#[tokio::test]
async fn caches_and_expires_host_geo() {
    let db_path = test_database_path("xrat-geoip-cache");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");

    verify_geoip_cache_state(&db).await;
}
