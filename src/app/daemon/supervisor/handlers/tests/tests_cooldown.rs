use super::super::health::should_record_health_failure;
use super::session_with_cooldown;

#[test]
fn suppresses_repeated_health_failure_while_cooldown_active() {
    let session = session_with_cooldown(Some(&(u64::MAX - 1).to_string()));
    assert!(!should_record_health_failure(&session));
}

#[test]
fn allows_health_failure_after_cooldown_expires() {
    let session = session_with_cooldown(Some("1"));
    assert!(should_record_health_failure(&session));
}
