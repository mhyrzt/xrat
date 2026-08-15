use crate::db::RuntimeSessionRecord;

mod tests_cooldown;
mod tests_health;
mod tests_replace;

fn session_with_cooldown(cooldown_until: Option<&str>) -> RuntimeSessionRecord {
    RuntimeSessionRecord {
        id: 1,
        config_id: Some(1),
        status: crate::db::RuntimeSessionStatus::Running,
        socks_host: Some("127.0.0.1".to_string()),
        socks_port: Some(1080),
        http_host: None,
        http_port: None,
        shadowsocks_host: None,
        shadowsocks_port: None,
        process_id: Some(1),
        failure_reason: None,
        owner_kind: Some("daemon".to_string()),
        owner_instance_id: Some("d1".to_string()),
        last_transition_reason_code: Some("health_check_failed".to_string()),
        last_transition_reason_detail: None,
        last_transition_origin: Some("daemon".to_string()),
        cooldown_until: cooldown_until.map(ToString::to_string),
        last_failed_at: Some("1".to_string()),
        last_failed_reason_code: Some("health_check_failed".to_string()),
        started_at: Some("1".to_string()),
        stopped_at: None,
        created_at: "1".to_string(),
        updated_at: "1".to_string(),
    }
}
