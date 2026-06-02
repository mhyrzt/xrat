use std::net::IpAddr;
use std::time::Duration;

use serde::Deserialize;

use super::{GeoIpError, GeoIpLookup};

#[cfg(test)]
mod tests;

const DEFAULT_IP_API_ENDPOINT: &str = "http://ip-api.com/json";

#[derive(Clone, Debug)]
pub struct RemoteIpApiLookup {
    client: reqwest::Client,
    endpoint: String,
}

impl RemoteIpApiLookup {
    pub fn new(endpoint: impl Into<String>, timeout: Duration) -> Result<Self, GeoIpError> {
        let endpoint = endpoint.into();
        let endpoint = if endpoint.trim().is_empty() {
            DEFAULT_IP_API_ENDPOINT.to_string()
        } else {
            endpoint.trim_end_matches('/').to_string()
        };

        let client = reqwest::Client::builder().timeout(timeout).build()?;
        Ok(Self { client, endpoint })
    }

    async fn fetch(&self, ip: IpAddr) -> Result<IpApiResponse, GeoIpError> {
        let ip_text = ip.to_string();
        let url = format!("{}/{}", self.endpoint, ip_text);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body_preview = response
                .text()
                .await
                .unwrap_or_default()
                .chars()
                .take(200)
                .collect();
            return Err(GeoIpError::Status {
                ip: ip_text,
                status,
                body_preview,
            });
        }

        let body = response
            .text()
            .await
            .map_err(|error| GeoIpError::Parse(error.to_string()))?;
        serde_json::from_str::<IpApiResponse>(&body)
            .map_err(|error| GeoIpError::Parse(error.to_string()))
    }
}

#[async_trait::async_trait]
impl GeoIpLookup for RemoteIpApiLookup {
    async fn country(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        Ok(self
            .fetch(ip)
            .await?
            .country_code
            .filter(|value| !value.is_empty()))
    }

    async fn city(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        let response = self.fetch(ip).await?;
        match (
            response.city.filter(|value| !value.is_empty()),
            response.region_name.filter(|value| !value.is_empty()),
            response.country_code.filter(|value| !value.is_empty()),
        ) {
            (Some(city), Some(region), Some(country)) => {
                Ok(Some(format!("{city}/{region}/{country}")))
            }
            (Some(city), _, Some(country)) => Ok(Some(format!("{city}/{country}"))),
            _ => Ok(None),
        }
    }

    async fn asn(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        let response = self.fetch(ip).await?;
        let Some(as_field) = response.as_field else {
            return Ok(None);
        };

        let mut parts = as_field.split_whitespace();
        let Some(asn) = parts.next() else {
            return Ok(None);
        };

        let org = response.org.unwrap_or_default();
        if org.trim().is_empty() {
            return Ok(Some(asn.to_string()));
        }

        Ok(Some(format!("{asn} {org}")))
    }

    fn backend_name(&self) -> &'static str {
        "ip-api"
    }
}

#[derive(Debug, Deserialize)]
struct IpApiResponse {
    #[serde(default, rename = "countryCode")]
    country_code: Option<String>,
    #[serde(default)]
    city: Option<String>,
    #[serde(default, rename = "regionName")]
    region_name: Option<String>,
    #[serde(default, rename = "as")]
    as_field: Option<String>,
    #[serde(default)]
    org: Option<String>,
}
