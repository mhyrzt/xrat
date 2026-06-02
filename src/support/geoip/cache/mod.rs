use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use super::{GeoIpError, GeoIpLookup};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct CachedLookup {
    inner: Arc<dyn GeoIpLookup>,
    ttl: Duration,
    max_entries: usize,
    entries: Mutex<HashMap<String, CacheEntry>>,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    value: Option<String>,
    expires_at: Instant,
    inserted_at: Instant,
}

impl CachedLookup {
    pub fn new(inner: Arc<dyn GeoIpLookup>, ttl: Duration, max_entries: usize) -> Self {
        Self {
            inner,
            ttl,
            max_entries,
            entries: Mutex::new(HashMap::new()),
        }
    }

    async fn lookup_or_fetch<F, Fut>(
        &self,
        aspect: &str,
        ip: IpAddr,
        fetch: F,
    ) -> Result<Option<String>, GeoIpError>
    where
        F: FnOnce(Arc<dyn GeoIpLookup>, IpAddr) -> Fut,
        Fut: std::future::Future<Output = Result<Option<String>, GeoIpError>>,
    {
        let key = format!("{aspect}:{ip}");
        if let Some(value) = self.get_cached(&key).await {
            return Ok(value);
        }

        let value = fetch(self.inner.clone(), ip).await?;
        self.insert_cached(key, value.clone()).await;
        Ok(value)
    }

    async fn get_cached(&self, key: &str) -> Option<Option<String>> {
        let mut entries = self.entries.lock().await;
        let now = Instant::now();
        match entries.get(key) {
            Some(entry) if entry.expires_at > now => Some(entry.value.clone()),
            Some(_) => {
                entries.remove(key);
                None
            }
            None => None,
        }
    }

    async fn insert_cached(&self, key: String, value: Option<String>) {
        let mut entries = self.entries.lock().await;
        let now = Instant::now();
        entries.insert(
            key,
            CacheEntry {
                value,
                expires_at: now + self.ttl,
                inserted_at: now,
            },
        );

        if entries.len() > self.max_entries
            && let Some(oldest_key) = entries
                .iter()
                .min_by_key(|(_, entry)| entry.inserted_at)
                .map(|(key, _)| key.clone())
        {
            entries.remove(&oldest_key);
        }
    }
}

#[async_trait::async_trait]
impl GeoIpLookup for CachedLookup {
    async fn country(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        self.lookup_or_fetch(
            "country",
            ip,
            |inner, ip| async move { inner.country(ip).await },
        )
        .await
    }

    async fn city(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        self.lookup_or_fetch("city", ip, |inner, ip| async move { inner.city(ip).await })
            .await
    }

    async fn asn(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        self.lookup_or_fetch("asn", ip, |inner, ip| async move { inner.asn(ip).await })
            .await
    }

    fn backend_name(&self) -> &'static str {
        self.inner.backend_name()
    }
}
