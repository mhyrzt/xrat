use super::super::geoip_cases::{test_args, test_runtime_paths};
use super::super::super::*;

use crate::app::config::{AppConfig, TestingSettings};

#[test]
fn resolves_custom_test_stage_order() {
    let app_config = AppConfig {
        testing: TestingSettings {
            order: vec![ConnectionTestStage::RealDelay, ConnectionTestStage::Icmp],
            ..TestingSettings::default()
        },
        ..AppConfig::default()
    };
    let runtime_paths = test_runtime_paths();

    let settings =
        resolve_test_settings(&test_args(Some(1)), &app_config, &runtime_paths).expect("settings");

    assert_eq!(
        settings.stage_order,
        vec![ConnectionTestStage::RealDelay, ConnectionTestStage::Icmp]
    );
}

#[test]
fn rejects_duplicate_test_stage_order_entries() {
    let app_config = AppConfig {
        testing: TestingSettings {
            order: vec![ConnectionTestStage::Icmp, ConnectionTestStage::Icmp],
            ..TestingSettings::default()
        },
        ..AppConfig::default()
    };
    let runtime_paths = test_runtime_paths();

    let error = resolve_test_settings(&test_args(Some(1)), &app_config, &runtime_paths)
        .expect_err("duplicate stage should fail");

    assert!(error.to_string().contains("duplicate test stage"));
}

#[test]
fn resolves_configured_failure_policy() {
    let app_config = AppConfig {
        testing: TestingSettings {
            failure_policy: TestFailurePolicy::SkipRemaining,
            ..TestingSettings::default()
        },
        ..AppConfig::default()
    };
    let runtime_paths = test_runtime_paths();

    let settings =
        resolve_test_settings(&test_args(Some(1)), &app_config, &runtime_paths).expect("settings");

    assert_eq!(settings.failure_policy, TestFailurePolicy::SkipRemaining);
    assert!(settings.failure_policy.halts_after_failure());
    assert!(TestFailurePolicy::MarkFailed.halts_after_failure());
    assert!(!TestFailurePolicy::Continue.halts_after_failure());
}
