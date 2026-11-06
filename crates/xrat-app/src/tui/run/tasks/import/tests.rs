use tempfile::TempDir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use super::{TuiImport, run_import};
use crate::app::context::AppContext;

#[tokio::test]
async fn tui_imports_named_xray_profile_from_local_file() {
    let (root, context) = test_context().await;
    let input = root.path().join("london.json");
    let bytes = include_bytes!("../../../../../../../testdata/xray/full-profile.json");
    std::fs::write(&input, bytes).unwrap();
    run_import(
        &context,
        TuiImport::Subscription {
            url: input.to_string_lossy().into_owned(),
            name: Some("london-vpn".into()),
            fallback_name: "Xray profile".into(),
        },
    )
    .await
    .unwrap();
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .pop()
        .unwrap();
    assert!(config.is_xray_profile());
    assert_eq!(config.name.as_deref(), Some("london-vpn"));
    assert_eq!(
        crate::app::services::xray_profiles::load(&context, &config)
            .await
            .unwrap()
            .as_bytes(),
        bytes
    );
}

async fn test_context() -> (TempDir, AppContext) {
    let (context, root) = crate::app::tests::TestAppBuilder::new("tui-import")
        .build_with_root()
        .await;
    (root, context)
}

async fn subscription_url(request_count: usize) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let address = listener.local_addr().expect("listener should have address");
    tokio::spawn(async move {
        let body = "vless://uuid-123@example.com:443#One";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        for _ in 0..request_count {
            let (mut socket, _) = listener.accept().await.expect("request should connect");
            // The request has to be drained before the socket is closed.
            // Windows answers an unread request with a reset, which would
            // abort the response the client is still reading.
            let mut request = Vec::new();
            let mut chunk = [0u8; 1024];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = socket
                    .read(&mut chunk)
                    .await
                    .expect("request should be readable");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..read]);
            }
            socket
                .write_all(response.as_bytes())
                .await
                .expect("response should be written");
        }
    });
    format!("http://{address}/subscription")
}

#[tokio::test]
async fn config_import_persists_one_config() {
    let (_root, context) = test_context().await;
    let (source, node) =
        crate::app::import::load_single_node("vless://uuid-123@example.com:443#One")
            .expect("config should parse");

    let message = run_import(
        &context,
        TuiImport::Config {
            source,
            node: Box::new(node),
        },
    )
    .await
    .expect("config import should succeed");

    assert_eq!(message, "added 1 config");
    assert_eq!(context.db.get_config_count().await.expect("count"), 1);
}

#[tokio::test]
async fn subscription_import_persists_name_and_reuses_url() {
    let (_root, context) = test_context().await;
    let url = subscription_url(2).await;

    for name in ["First", "Second"] {
        run_import(
            &context,
            TuiImport::Subscription {
                url: url.clone(),
                name: Some(name.to_string()),
                fallback_name: "sub-fallback".into(),
            },
        )
        .await
        .expect("subscription import should succeed");
    }

    let subscriptions = context
        .db
        .list_subscriptions()
        .await
        .expect("subscriptions should load");
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0].name.as_deref(), Some("Second"));
    assert_eq!(context.db.get_config_count().await.expect("count"), 1);
}

#[tokio::test]
async fn automatic_tui_name_uses_server_precedence_and_preserves_explicit_names() {
    let (_root, context) = test_context().await;
    let (url, server) = crate::app::tests::fixtures::named_subscription_url(2).await;
    for (name, expected) in [
        (None, "ServerProvider"),
        (Some("sub-work".into()), "sub-work"),
    ] {
        let message = run_import(
            &context,
            TuiImport::Subscription {
                url: url.clone(),
                name,
                fallback_name: "UrlProvider".into(),
            },
        )
        .await
        .unwrap();
        let stored = context.db.list_subscriptions().await.unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].name.as_deref(), Some(expected));
        assert!(message.ends_with(expected));
    }
    server.await.unwrap();
}

#[tokio::test]
async fn automatic_tui_name_uses_fallback_only_when_metadata_is_absent() {
    let (_root, context) = test_context().await;
    let url = subscription_url(1).await;
    run_import(
        &context,
        TuiImport::Subscription {
            url,
            name: None,
            fallback_name: "sub-fallback".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(
        context.db.list_subscriptions().await.unwrap()[0]
            .name
            .as_deref(),
        Some("sub-fallback")
    );
}
