use super::*;

pub(super) async fn verify_import_and_config_state(db: &Database) -> (i64, i64) {
    let source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.com/sub".to_string(),
        name: Some("Example".to_string()),
    };
    let first = test_node("first");
    let mut second = test_node("second");
    second.address = "second.example.com".to_string();
    second.uuid = Some("uuid-456".to_string());
    second.raw_config =
        "vless://uuid-456@second.example.com:443?type=ws&security=tls#second".to_string();

    let summary = db
        .import_nodes(&source, &[first, second])
        .await
        .expect("import should succeed");
    assert_eq!(summary.imported_configs, 2);
    assert_eq!(summary.total_configs, 2);
    assert_eq!(db.get_subscription_count().await.expect("count"), 1);
    assert_eq!(db.get_config_count().await.expect("count"), 2);

    let subscriptions = db.list_subscriptions().await.expect("subscriptions");
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0].source_kind, "url");
    assert_eq!(subscriptions[0].config_count, 2);

    let configs = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("configs should load");
    let first_id = configs[0].id;
    let second_id = configs[1].id;

    db.set_active_config(first_id)
        .await
        .expect("activate first should succeed");
    db.set_active_config(second_id)
        .await
        .expect("activate second should succeed");

    let active = db
        .get_active_config()
        .await
        .expect("active query should succeed")
        .expect("active config should exist");
    assert_eq!(active.id, second_id);

    db.set_config_enabled(second_id, false)
        .await
        .expect("disable should succeed");
    let enabled_configs = db
        .list_configs(&ConfigListFilter {
            only_enabled: true,
            ..ConfigListFilter::default()
        })
        .await
        .expect("enabled configs should load");
    assert_eq!(enabled_configs.len(), 1);
    assert!(db.get_active_config().await.expect("active").is_none());

    db.set_config_enabled(second_id, true)
        .await
        .expect("enable should succeed");
    db.delete_config(first_id)
        .await
        .expect("delete should succeed");

    (first_id, second_id)
}

pub(super) async fn verify_reconcile_state(db: &Database) {
    let source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.com/reconcile".to_string(),
        name: None,
    };
    let mut first = test_node("recon-a");
    first.address = "recon-a.example.com".to_string();
    first.uuid = Some("uuid-recon-a".to_string());
    first.raw_config =
        "vless://uuid-recon-a@recon-a.example.com:443?type=ws&security=tls#a".to_string();
    let mut second = test_node("recon-b");
    second.address = "recon-b.example.com".to_string();
    second.uuid = Some("uuid-recon-b".to_string());
    second.raw_config =
        "vless://uuid-recon-b@recon-b.example.com:443?type=ws&security=tls#b".to_string();

    let initial = db
        .import_nodes(&source, &[first.clone(), second])
        .await
        .expect("reconcile import should succeed");
    assert_eq!(initial.removed_configs, 0);
    let subscription_id = initial.subscription_id;
    let active_filter = ConfigListFilter {
        subscription_id: Some(subscription_id),
        ..ConfigListFilter::default()
    };
    assert_eq!(
        db.list_configs(&active_filter).await.expect("list").len(),
        2
    );

    let refreshed = db
        .import_nodes(&source, &[first])
        .await
        .expect("reconcile refresh should succeed");
    assert_eq!(refreshed.removed_configs, 1);
    assert_eq!(
        db.list_configs(&active_filter).await.expect("list").len(),
        1
    );

    let all_filter = ConfigListFilter {
        subscription_id: Some(subscription_id),
        include_deleted: true,
        ..ConfigListFilter::default()
    };
    assert_eq!(
        db.list_configs(&all_filter).await.expect("list all").len(),
        2,
        "removed config is soft-deleted, not purged"
    );
}
