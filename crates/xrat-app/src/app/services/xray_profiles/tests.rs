use super::*;
use crate::app::tests::TestAppBuilder;

const PROFILE: &[u8] = include_bytes!("../../../../../../testdata/xray/full-profile.json");

async fn import_profile(context: &AppContext) -> xrat_db::ImportSummary {
    let source = xrat_db::ImportSource {
        kind: xrat_db::SourceKind::File,
        value: "/tmp/original.json".into(),
        name: None,
    };
    import(
        context,
        &source,
        &parse(PROFILE).unwrap().unwrap(),
        Some("london-vpn"),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn import_preserves_complete_document_deduplicates_and_restores_deleted_profile() {
    let (context, _root) = TestAppBuilder::new("xray-profile-import")
        .build_with_root()
        .await;
    let first = import_profile(&context).await;
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .pop()
        .unwrap();
    assert!(config.is_xray_profile());
    assert_eq!(config.raw_config.as_bytes(), PROFILE);
    assert_eq!(config.name.as_deref(), Some("london-vpn"));
    assert_eq!(load(&context, &config).await.unwrap().as_bytes(), PROFILE);
    context.db.delete_config(config.id).await.unwrap();
    let second = import_profile(&context).await;
    assert_eq!(first.subscription_id, second.subscription_id);
    let configs = context.db.list_configs(&Default::default()).await.unwrap();
    assert_eq!(configs.len(), 1);
    assert_eq!(context.db.list_subscriptions().await.unwrap().len(), 1);
    assert_eq!(configs[0].r#ref, config.r#ref);
    assert_eq!(
        load(&context, &configs[0]).await.unwrap().as_bytes(),
        PROFILE
    );
    assert_eq!(configs[0].raw_config.as_bytes(), PROFILE);
    assert!(
        context
            .db
            .get_xray_profile(config.id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(!ProfileStore::for_context(&context).root.exists());
}

#[tokio::test]
async fn corrupt_database_profile_fails_closed() {
    let (context, _root) = TestAppBuilder::new("xray-profile-integrity")
        .build_with_root()
        .await;
    import_profile(&context).await;
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .pop()
        .unwrap();
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite:{}",
        context.runtime_paths.database_path.display()
    ))
    .await
    .unwrap();
    sqlx::query("UPDATE configs SET raw_config = ?1 WHERE id = ?2")
        .bind("{}")
        .bind(config.id)
        .execute(&pool)
        .await
        .unwrap();
    let corrupted = context
        .db
        .get_config_by_id(config.id)
        .await
        .unwrap()
        .unwrap();
    assert!(
        load(&context, &corrupted)
            .await
            .unwrap_err()
            .to_string()
            .contains("integrity")
    );
}

#[tokio::test]
async fn legacy_file_profile_is_copied_into_database_on_first_load() {
    let (context, _root) = TestAppBuilder::new("xray-profile-legacy-migration")
        .build_with_root()
        .await;
    import_profile(&context).await;
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .pop()
        .unwrap();
    let key = digest(PROFILE);
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite:{}",
        context.runtime_paths.database_path.display()
    ))
    .await
    .unwrap();
    sqlx::query("UPDATE configs SET raw_config = '' WHERE id = ?1")
        .bind(config.id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO xray_profiles (config_id, digest, byte_length) VALUES (?1, ?2, ?3)")
        .bind(config.id)
        .bind(&key)
        .bind(PROFILE.len() as i64)
        .execute(&pool)
        .await
        .unwrap();
    let store = ProfileStore::for_context(&context);
    {
        let _lock = store.lock().unwrap();
        fs::write(store.path(&key).unwrap(), PROFILE).unwrap();
    }
    let legacy = context
        .db
        .get_config_by_id(config.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(load(&context, &legacy).await.unwrap().as_bytes(), PROFILE);
    let migrated = context
        .db
        .get_config_by_id(config.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(migrated.raw_config.as_bytes(), PROFILE);
    assert!(
        context
            .db
            .get_xray_profile(config.id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(!store.path(&key).unwrap().exists());
}

#[tokio::test]
async fn failed_database_import_rolls_back_catalog_without_creating_profile_files() {
    let (context, _root) = TestAppBuilder::new("xray-profile-rollback")
        .build_with_root()
        .await;
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite:{}",
        context.runtime_paths.database_path.display()
    ))
    .await
    .unwrap();
    sqlx::query("CREATE TRIGGER reject_profile BEFORE INSERT ON configs WHEN NEW.protocol = 'xray-json' BEGIN SELECT RAISE(FAIL, 'injected failure'); END").execute(&pool).await.unwrap();
    let source = xrat_db::ImportSource {
        kind: xrat_db::SourceKind::RawText,
        value: "test".into(),
        name: None,
    };
    assert!(
        import(&context, &source, &parse(PROFILE).unwrap().unwrap(), None)
            .await
            .is_err()
    );
    assert!(
        context
            .db
            .list_configs(&Default::default())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(context.db.list_subscriptions().await.unwrap().is_empty());
    assert!(!ProfileStore::for_context(&context).root.exists());
}

#[test]
fn malformed_or_wrong_json_does_not_become_an_empty_subscription() {
    assert!(parse(b"{broken").is_err());
    assert!(parse(b"[]").is_err());
    assert!(parse(br#"{"servers":[]}"#).is_err());
    assert!(parse(b"vless://example").unwrap().is_none());
}

#[tokio::test]
async fn catalog_export_and_lifecycle_preserve_profile_until_permanent_deletion() {
    let (context, _root) = TestAppBuilder::new("xray-profile-lifecycle")
        .build_with_root()
        .await;
    import_profile(&context).await;
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .pop()
        .unwrap();
    let services = context.services();
    let raw = services
        .configs
        .raw_configs_by_ids(&[config.id])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(raw[0].as_bytes(), PROFILE);
    assert!(
        services
            .configs
            .export_raw_configs(&Default::default())
            .await
            .is_err()
    );
    let reference = xrat_model::ConfigRef::from(config.r#ref.clone());
    services.lifecycle.delete(&reference, false).await.unwrap();
    assert_eq!(load(&context, &config).await.unwrap().as_bytes(), PROFILE);
    services.lifecycle.restore(&reference).await.unwrap();
    services.lifecycle.delete(&reference, true).await.unwrap();
    assert!(
        context
            .db
            .get_xray_profile(config.id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(!ProfileStore::for_context(&context).root.exists());
}

#[tokio::test]
async fn cli_import_of_named_file_remains_usable_after_original_is_removed() {
    let (context, root) = TestAppBuilder::new("xray-profile-cli")
        .build_with_root()
        .await;
    let original = root.path().join("london.json");
    fs::write(&original, PROFILE).unwrap();
    crate::app::commands::run(
        &context,
        &crate::cli::Command::Import(crate::cli::ImportArgs {
            input: original.to_string_lossy().into_owned(),
            url_list: false,
            name: Some("london-vpn".into()),
            headers: Vec::new(),
        }),
    )
    .await
    .unwrap();
    fs::remove_file(original).unwrap();
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(config.name.as_deref(), Some("london-vpn"));
    assert_eq!(load(&context, &config).await.unwrap().as_bytes(), PROFILE);
}
