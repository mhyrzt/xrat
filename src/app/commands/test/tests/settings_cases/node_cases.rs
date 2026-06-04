use super::super::super::*;

#[test]
fn rebuilds_node_from_config_record() {
    let record = ConfigRecord {
        id: 1,
        subscription_id: Some(2),
        dedup_key: "key".to_string(),
        protocol: "vmess".to_string(),
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: Some("uuid-123".to_string()),
        password: None,
        method: None,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("cdn.example.com".to_string()),
        host: Some("cdn.example.com".to_string()),
        path: Some("/socket".to_string()),
        name: Some("node".to_string()),
        raw_config: "vmess://payload".to_string(),
        is_active: false,
        is_enabled: true,
        is_deleted: false,
        deleted_at: None,
        imported_at: "now".to_string(),
        created_at: "now".to_string(),
        updated_at: "now".to_string(),
    };

    let node = node_from_record(&record).expect("config record should rebuild");
    assert_eq!(node.protocol.as_str(), "vmess");
    assert_eq!(node.address, "example.com");
    assert_eq!(node.network, "ws");
    assert_eq!(node.uuid.as_deref(), Some("uuid-123"));
}
