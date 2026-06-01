use crate::tui::data::{TuiConfigRow, TuiSourceRow};

pub(crate) fn row(id: i64) -> TuiConfigRow {
    TuiConfigRow {
        id,
        name: format!("config-{id}"),
        protocol: "vless".to_string(),
        address: "example.com".to_string(),
        port: 443,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        real_delay_ms: Some(100),
        tcp_ms: Some(20),
        failure_reason: None,
        source_id: None,
        is_active: false,
        is_enabled: true,
        is_selected: false,
        is_deleted: false,
    }
}

pub(crate) fn source(id: i64) -> TuiSourceRow {
    TuiSourceRow {
        id,
        kind: "url".to_string(),
        value: format!("https://example.com/{id}"),
        name: Some(format!("source-{id}")),
        config_count: id,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    }
}
