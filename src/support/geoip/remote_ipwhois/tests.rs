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
    let server = spawn_json_server(StatusCode::TOO_MANY_REQUESTS, "rate limited".to_string()).await;
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
