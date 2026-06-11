//! xray/v2ray StatsService client. The service is gRPC-only and is exposed by
//! the `api` dokodemo-door inbound that `enable_stats_api` adds. The channel is
//! plaintext h2c to localhost, so no TLS backend is pulled in. The prost
//! messages are hand-written to avoid a `build.rs`/`protoc` codegen step.

use tonic::IntoRequest;
use tonic::codec::ProstCodec;
use tonic::codegen::http::uri::PathAndQuery;
use tonic::transport::Channel;

use super::{StatsError, StatsSample, StatsSource};

const QUERY_STATS_PATH: &str = "/xray.app.stats.command.StatsService/QueryStats";

#[derive(Clone, PartialEq, prost::Message)]
struct QueryStatsRequest {
    #[prost(string, tag = "1")]
    pattern: String,
    #[prost(bool, tag = "2")]
    reset: bool,
}

#[derive(Clone, PartialEq, prost::Message)]
struct Stat {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(int64, tag = "2")]
    value: i64,
}

#[derive(Clone, PartialEq, prost::Message)]
struct QueryStatsResponse {
    #[prost(message, repeated, tag = "1")]
    stat: Vec<Stat>,
}

/// Samples xray traffic counters over the StatsService gRPC endpoint. `endpoint`
/// is an `http://host:port` URI pointing at the api inbound.
pub struct XrayStatsSource {
    endpoint: String,
}

impl XrayStatsSource {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            endpoint: format!("http://{host}:{port}"),
        }
    }
}

#[async_trait::async_trait]
impl StatsSource for XrayStatsSource {
    async fn sample(&self) -> Result<StatsSample, StatsError> {
        let channel = Channel::from_shared(self.endpoint.clone())
            .map_err(|error| StatsError(error.to_string()))?
            .connect()
            .await
            .map_err(|error| StatsError(error.to_string()))?;

        let mut client = tonic::client::Grpc::new(channel);
        client
            .ready()
            .await
            .map_err(|error| StatsError(error.to_string()))?;

        let request = QueryStatsRequest {
            pattern: String::new(),
            reset: false,
        }
        .into_request();
        let codec: ProstCodec<QueryStatsRequest, QueryStatsResponse> = ProstCodec::default();
        let path = PathAndQuery::from_static(QUERY_STATS_PATH);
        let response = client
            .unary(request, path, codec)
            .await
            .map_err(|status| StatsError(status.to_string()))?;

        Ok(sum_outbound_traffic(&response.into_inner()))
    }
}

/// Sum every outbound `uplink`/`downlink` counter into session totals. xray stat
/// names look like `outbound>>>proxy>>>traffic>>>uplink`; inbound counters are
/// ignored so the totals reflect traffic leaving through the proxies.
fn sum_outbound_traffic(response: &QueryStatsResponse) -> StatsSample {
    let mut uplink_total = 0u64;
    let mut downlink_total = 0u64;
    for stat in &response.stat {
        if !stat.name.starts_with("outbound>>>") {
            continue;
        }
        let value = stat.value.max(0) as u64;
        match stat.name.rsplit(">>>").next() {
            Some("uplink") => uplink_total += value,
            Some("downlink") => downlink_total += value,
            _ => {}
        }
    }
    StatsSample {
        uplink_total,
        downlink_total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_only_outbound_counters() {
        let response = QueryStatsResponse {
            stat: vec![
                Stat {
                    name: "outbound>>>proxy>>>traffic>>>uplink".to_string(),
                    value: 100,
                },
                Stat {
                    name: "outbound>>>proxy>>>traffic>>>downlink".to_string(),
                    value: 900,
                },
                Stat {
                    name: "outbound>>>direct>>>traffic>>>uplink".to_string(),
                    value: 5,
                },
                Stat {
                    name: "inbound>>>socks-in>>>traffic>>>uplink".to_string(),
                    value: 7777,
                },
            ],
        };
        let sample = sum_outbound_traffic(&response);
        assert_eq!(sample.uplink_total, 105);
        assert_eq!(sample.downlink_total, 900);
    }
}
