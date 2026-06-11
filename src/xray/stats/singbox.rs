//! sing-box traffic stats over the Clash API controller exposed by
//! `experimental.clash_api`. The `/connections` endpoint reports cumulative
//! `uploadTotal`/`downloadTotal` byte counters for the session, which map
//! directly onto [`StatsSample`].

use serde::Deserialize;

use super::{StatsError, StatsSample, StatsSource};

#[derive(Debug, Deserialize)]
struct ClashConnections {
    #[serde(rename = "uploadTotal", default)]
    upload_total: u64,
    #[serde(rename = "downloadTotal", default)]
    download_total: u64,
}

/// Samples sing-box traffic counters via the Clash API `/connections` endpoint.
/// `controller` is the `host:port` external controller; `secret` is the optional
/// bearer token configured on the controller.
pub struct SingboxStatsSource {
    url: String,
    secret: Option<String>,
    client: reqwest::Client,
}

impl SingboxStatsSource {
    pub fn new(controller: &str, secret: Option<String>) -> Self {
        Self {
            url: format!("http://{controller}/connections"),
            secret,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl StatsSource for SingboxStatsSource {
    async fn sample(&self) -> Result<StatsSample, StatsError> {
        let mut request = self.client.get(&self.url);
        if let Some(secret) = &self.secret {
            request = request.bearer_auth(secret);
        }
        let body = request
            .send()
            .await
            .map_err(|error| StatsError(error.to_string()))?
            .text()
            .await
            .map_err(|error| StatsError(error.to_string()))?;
        let connections: ClashConnections =
            serde_json::from_str(&body).map_err(|error| StatsError(error.to_string()))?;
        Ok(StatsSample {
            uplink_total: connections.upload_total,
            downlink_total: connections.download_total,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clash_connection_totals() {
        let parsed: ClashConnections = serde_json::from_str(
            r#"{"downloadTotal": 2048, "uploadTotal": 512, "connections": [], "memory": 1}"#,
        )
        .unwrap();
        assert_eq!(parsed.upload_total, 512);
        assert_eq!(parsed.download_total, 2048);
    }
}
