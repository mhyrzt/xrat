use crate::tui::data::{TuiConfigRow, TuiSourceRow};

pub(crate) fn row(id: i64) -> TuiConfigRow {
    TuiConfigRow {
        id,
        r#ref: format!("ref{id:09}"),
        name: format!("config-{id}"),
        protocol: "vless".to_string(),
        address: "example.com".to_string(),
        port: 443,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        icmp_ms: Some(10),
        real_delay_ms: Some(100),
        tcp_ms: Some(20),
        download_mbps: Some(42.25),
        upload_mbps: Some(11.5),
        dial_endpoint_country: Some("NL".to_string()),
        dial_endpoint_location: Some("NL/North Holland/Amsterdam".to_string()),
        dial_endpoint_asn: Some("AS60781 LeaseWeb".to_string()),
        dial_endpoint_fronting: None,
        failure_reason: None,
        source_id: None,
        tested_at: Some("2026-01-01T00:00:00Z".to_string()),
        imported_at: "2026-01-01T00:00:00Z".to_string(),
        is_active: false,
        is_enabled: true,
        is_deleted: false,
    }
}

pub(crate) fn source(id: i64) -> TuiSourceRow {
    TuiSourceRow {
        id,
        r#ref: format!("src{id:09}"),
        kind: "url".to_string(),
        value: format!("https://example.com/{id}"),
        name: Some(format!("source-{id}")),
        config_count: id,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    }
}
