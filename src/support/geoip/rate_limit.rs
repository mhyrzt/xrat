use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;

use super::{GeoIpError, GeoIpLookup};

#[derive(Debug)]
pub struct RateLimitedLookup {
    inner: Arc<dyn GeoIpLookup>,
    budget_per_window: u32,
    window: Duration,
    state: Mutex<RateLimitState>,
}

#[derive(Debug)]
struct RateLimitState {
    window_started_at: Instant,
    used: u32,
}

impl RateLimitedLookup {
    pub fn new(inner: Arc<dyn GeoIpLookup>, budget_per_window: u32, window: Duration) -> Self {
        Self {
            inner,
            budget_per_window,
            window,
            state: Mutex::new(RateLimitState {
                window_started_at: Instant::now(),
                used: 0,
            }),
        }
    }

    async fn check_budget(&self) -> Result<(), GeoIpError> {
        let mut state = self.state.lock().await;
        let now = Instant::now();
        if now.duration_since(state.window_started_at) >= self.window {
            state.window_started_at = now;
            state.used = 0;
        }

        if state.used >= self.budget_per_window {
            let retry_after = self
                .window
                .saturating_sub(now.duration_since(state.window_started_at))
                .as_secs();
            return Err(GeoIpError::RateLimited {
                retry_after_secs: retry_after.max(1),
            });
        }

        state.used += 1;
        Ok(())
    }
}

#[async_trait::async_trait]
impl GeoIpLookup for RateLimitedLookup {
    async fn country(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        self.check_budget().await?;
        self.inner.country(ip).await
    }

    async fn city(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        self.check_budget().await?;
        self.inner.city(ip).await
    }

    async fn asn(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        self.check_budget().await?;
        self.inner.asn(ip).await
    }

    fn backend_name(&self) -> &'static str {
        self.inner.backend_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestLookup;

    #[async_trait::async_trait]
    impl GeoIpLookup for TestLookup {
        async fn country(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
            Ok(Some("NL".to_string()))
        }

        async fn city(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
            Ok(Some("Amsterdam/NL".to_string()))
        }

        async fn asn(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
            Ok(Some("AS1 Example".to_string()))
        }

        fn backend_name(&self) -> &'static str {
            "test"
        }
    }

    #[tokio::test]
    async fn rejects_requests_over_budget() {
        let lookup = RateLimitedLookup::new(Arc::new(TestLookup), 1, Duration::from_secs(60));
        let ip: IpAddr = "8.8.8.8".parse().unwrap();

        assert_eq!(lookup.country(ip).await.unwrap().as_deref(), Some("NL"));
        match lookup
            .country(ip)
            .await
            .expect_err("second call should rate limit")
        {
            GeoIpError::RateLimited { retry_after_secs } => assert!(retry_after_secs >= 1),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
