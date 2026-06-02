use std::net::IpAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::super::{GeoIpError, GeoIpLookup};
use super::CachedLookup;

#[derive(Debug)]
struct TestLookup {
    calls: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl GeoIpLookup for TestLookup {
    async fn country(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Some("NL".to_string()))
    }

    async fn city(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Some("Amsterdam/NL".to_string()))
    }

    async fn asn(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Some("AS1 Example".to_string()))
    }

    fn backend_name(&self) -> &'static str {
        "test"
    }
}

#[tokio::test]
async fn caches_repeated_country_lookups() {
    let calls = Arc::new(AtomicUsize::new(0));
    let lookup = CachedLookup::new(
        Arc::new(TestLookup {
            calls: calls.clone(),
        }),
        Duration::from_secs(60),
        10,
    );
    let ip: IpAddr = "8.8.8.8".parse().unwrap();

    assert_eq!(lookup.country(ip).await.unwrap().as_deref(), Some("NL"));
    assert_eq!(lookup.country(ip).await.unwrap().as_deref(), Some("NL"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
