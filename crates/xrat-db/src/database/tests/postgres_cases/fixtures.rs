use super::*;

pub(in super::super) async fn verify_database_backend(db: &Database) {
    let (first_id, second_id) = config_cases::verify_import_and_config_state(db).await;
    config_cases::verify_reconcile_state(db).await;
    super::super::import_cases::verify_refresh_purge(db).await;
    connection_test_cases::verify_connection_test_state(db, second_id).await;
    runtime_session_cases::verify_runtime_session_state(db, second_id).await;
    super::super::import_cases::verify_subscription_headers(db).await;
    super::super::geoip_cache_cases::verify_geoip_cache_state(db).await;
    verify_xray_profile_state(db).await;

    let deleted = db
        .get_config_by_id(first_id)
        .await
        .expect("deleted query")
        .expect("soft-deleted config remains stored");
    assert!(deleted.is_deleted);
    assert!(!deleted.is_active);
    db.hard_delete_config(first_id).await.expect("purge config");
    assert!(
        db.get_config_by_id(first_id)
            .await
            .expect("purged query")
            .is_none()
    );
}

async fn verify_xray_profile_state(db: &Database) {
    let digest = "a".repeat(64);
    let source = crate::ImportSource {
        kind: crate::SourceKind::RawText,
        value: format!("xray-profile:{digest}"),
        name: Some("raw fixture".into()),
    };
    let first = db
        .import_xray_profile(&source, &digest, 123, Some("London"))
        .await
        .unwrap();
    let config = db
        .list_configs(&crate::ConfigListFilter {
            protocol: Some(crate::XRAY_PROFILE_PROTOCOL.into()),
            ..Default::default()
        })
        .await
        .unwrap()
        .pop()
        .unwrap();
    assert!(config.is_xray_profile());
    assert!(config.raw_config.is_empty());
    assert!(crate::node_from_record(&config).is_err());
    let metadata = db.get_xray_profile(config.id).await.unwrap().unwrap();
    assert_eq!(
        (metadata.digest, metadata.byte_length),
        (digest.clone(), 123)
    );
    db.delete_config(config.id).await.unwrap();
    assert!(db.get_xray_profile(config.id).await.unwrap().is_some());
    let second = db
        .import_xray_profile(&source, &digest, 123, Some("Renamed"))
        .await
        .unwrap();
    assert_eq!(first.subscription_id, second.subscription_id);
    let restored = db.get_config_by_id(config.id).await.unwrap().unwrap();
    assert!(!restored.is_deleted);
    assert_eq!(restored.r#ref, config.r#ref);
    assert_eq!(restored.name.as_deref(), Some("Renamed"));
    db.hard_delete_config(config.id).await.unwrap();
    assert!(db.get_xray_profile(config.id).await.unwrap().is_none());
}
