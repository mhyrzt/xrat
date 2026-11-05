use super::*;

#[tokio::test]
async fn browser_import_preserves_json_rejection_and_does_not_read_server_files() {
    let (context, root) = app::tests::TestAppBuilder::new("web-import")
        .build_with_root()
        .await;
    std::fs::write(&context.runtime_paths.config_path, "").unwrap();
    let flag = xrat_support::cancel::CancellationFlag::new();
    let result = execute(
        &context,
        Action::Import {
            input: "{\"outbounds\":[]}".into(),
            name: None,
        },
        &flag,
        "test",
    )
    .await;
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("local CLI/TUI import")
    );
    let file = root.path().join("private-subscription.txt");
    std::fs::write(
        &file,
        "vless://11111111-1111-1111-1111-111111111111@example.test:443?security=tls",
    )
    .unwrap();
    let result = execute(
        &context,
        Action::Import {
            input: file.display().to_string(),
            name: None,
        },
        &flag,
        "test",
    )
    .await;
    assert!(matches!(result, Err(app::AppError::NoSupportedConfig)));
    assert!(
        context
            .db
            .list_configs(&Default::default())
            .await
            .unwrap()
            .is_empty()
    );
}
