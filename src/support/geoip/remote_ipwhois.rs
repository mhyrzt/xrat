use std::net::IpAddr;
use std::time::Duration;

use serde::Deserialize;

use super::{GeoIpError, GeoIpLookup};

const DEFAULT_IPWHOIS_ENDPOINT: &str = "https://ipwhois.app/json";

#[derive(Clone, Debug)]
pub struct RemoteIpWhoisLookup {
    client: reqwest::Client,
    endpoint: String,
}

impl RemoteIpWhoisLookup {
    pub fn new(endpoint: impl Into<String>, timeout: Duration) -> Result<Self, GeoIpError> {
        let endpoint = endpoint.into();
        let endpoint = if endpoint.trim().is_empty() {
            DEFAULT_IPWHOIS_ENDPOINT.to_string()
        } else {
            endpoint.trim_end_matches('/').to_string()
        };

        let client = reqwest::Client::builder().timeout(timeout).build()?;
        Ok(Self { client, endpoint })
    }

    async fn fetch(&self, ip: IpAddr) -> Result<IpWhoisResponse, GeoIpError> {
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
        serde_json::from_str::<IpWhoisResponse>(&body)
            .map_err(|error| GeoIpError::Parse(error.to_string()))
    }
}

#[async_trait::async_trait]
impl GeoIpLookup for RemoteIpWhoisLookup {
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
            response.country_code.filter(|value| !value.is_empty()),
        ) {
            (Some(city), Some(country)) => Ok(Some(format!("{city}/{country}"))),
            _ => Ok(None),
        }
    }

    async fn asn(&self, ip: IpAddr) -> Result<Option<String>, GeoIpError> {
        let response = self.fetch(ip).await?;
        let Some(connection) = response.connection else {
            return Ok(None);
        };
        let Some(asn) = connection.asn else {
            return Ok(None);
        };

        let org = connection.org.unwrap_or_default();
        if org.trim().is_empty() {
            return Ok(Some(format!("AS{asn}")));
        }

        Ok(Some(format!("AS{asn} {org}")))
    }

    fn backend_name(&self) -> &'static str {
        "ipwhois"
    }
}

#[derive(Debug, Deserialize)]
struct IpWhoisResponse {
    #[serde(default)]
    country_code: Option<String>,
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    connection: Option<IpWhoisConnection>,
}

#[derive(Debug, Deserialize)]
struct IpWhoisConnection {
    #[serde(default)]
    asn: Option<u64>,
    #[serde(default)]
    org: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, http::StatusCode, routing::get};
    use serde_json::json;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn resolves_country_city_and_asn_from_stubbed_service() {
        let server = spawn_json_server(
            StatusCode::OK,
            json!({
                "country_code": "NL",
                "city": "Amsterdam",
                "connection": { "asn": 60781, "org": "LeaseWeb" }
            })
            .to_string(),
        )
        .await;
        let lookup = RemoteIpWhoisLookup::new(server, Duration::from_secs(1)).unwrap();
        let ip: IpAddr = "8.8.8.8".parse().unwrap();

        assert_eq!(lookup.country(ip).await.unwrap().as_deref(), Some("NL"));
        assert_eq!(
            lookup.city(ip).await.unwrap().as_deref(),
            Some("Amsterdam/NL")
        );
        assert_eq!(
            lookup.asn(ip).await.unwrap().as_deref(),
            Some("AS60781 LeaseWeb")
        );
    }

    #[tokio::test]
    async fn reports_http_status_errors() {
        let server =
            spawn_json_server(StatusCode::TOO_MANY_REQUESTS, "rate limited".to_string()).await;
        let lookup = RemoteIpWhoisLookup::new(server, Duration::from_secs(1)).unwrap();
        let ip: IpAddr = "8.8.8.8".parse().unwrap();

        match lookup.country(ip).await.expect_err("status should fail") {
            GeoIpError::Status { status, .. } => assert_eq!(status, 429),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[tokio::test]
    async fn reports_parse_errors_for_malformed_json() {
        let server = spawn_json_server(StatusCode::OK, "not-json".to_string()).await;
        let lookup = RemoteIpWhoisLookup::new(server, Duration::from_secs(1)).unwrap();
        let ip: IpAddr = "8.8.8.8".parse().unwrap();

        match lookup.country(ip).await.expect_err("parse should fail") {
            GeoIpError::Parse(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    async fn spawn_json_server(status: StatusCode, body: String) -> String {
        let app = Router::new().route(
            "/json/{ip}",
            get(move || {
                let body = body.clone();
                async move { (status, body) }
            }),
        );

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        format!("http://{}/json", address)
    }
}
