use std::net::IpAddr;
use std::sync::Arc;

use super::{GeoIpError, GeoIpLookup};

#[derive(Debug)]
pub struct ChainedLookup {
    primary: Arc<dyn GeoIpLookup>,
    fallback: Arc<dyn GeoIpLookup>,
}

impl ChainedLookup {
    pub fn new(primary: Arc<dyn GeoIpLookup>, fallback: Arc<dyn GeoIpLookup>) -> Self {
        Self { primary, fallback }
    }
}

#[async_trait::async_trait]
impl GeoIpLookup for ChainedLookup {
    async fn country(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        match self.primary.country(ip).await? {
            Some(value) => Ok(Some(value)),
            None => self.fallback.country(ip).await,
        }
    }

    async fn city(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        match self.primary.city(ip).await? {
            Some(value) => Ok(Some(value)),
            None => self.fallback.city(ip).await,
        }
    }

    async fn asn(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        match self.primary.asn(ip).await? {
            Some(value) => Ok(Some(value)),
            None => self.fallback.asn(ip).await,
        }
    }

    fn backend_name(&self) -> &'static str {
        self.primary.backend_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestLookup {
        value: Option<&'static str>,
        name: &'static str,
    }

    #[async_trait::async_trait]
    impl GeoIpLookup for TestLookup {
        async fn country(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
            Ok(self.value.map(str::to_string))
        }

        async fn city(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
            Ok(self.value.map(str::to_string))
        }

        async fn asn(&self, _ip: IpAddr) -> Result<Option<String>, GeoIpError> {
            Ok(self.value.map(str::to_string))
        }

        fn backend_name(&self) -> &'static str {
            self.name
        }
    }

    #[tokio::test]
    async fn falls_back_when_primary_returns_none() {
        let chain = ChainedLookup::new(
            Arc::new(TestLookup {
                value: None,
                name: "mmdb",
            }),
            Arc::new(TestLookup {
                value: Some("NL"),
                name: "ipwhois",
            }),
        );

        let ip: IpAddr = "8.8.8.8".parse().unwrap();
        assert_eq!(chain.country(ip).await.unwrap().as_deref(), Some("NL"));
    }
}
