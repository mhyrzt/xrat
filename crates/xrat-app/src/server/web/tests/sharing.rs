use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use xrat_web::configs::sharing::{ConfigFilter, ConfigShareRequest, ConfigShareResponse};

#[tokio::test]
async fn raw_xray_profile_download_requires_authorization_and_is_not_a_subscription() {
    let (router, state) = fixture().await;
    let context = state.web.as_ref().unwrap().context.as_ref().unwrap();
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/xray/full-profile.json"
    ));
    crate::app::services::catalog_import::persist_content(
        context,
        xrat_db::ImportSource {
            kind: xrat_db::SourceKind::RawText,
            value: "fixture".into(),
            name: None,
        },
        bytes,
        Some("london-vpn"),
        &Default::default(),
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
    let path = format!("/api/v1/configs/{}/raw", config.id);
    assert_eq!(
        request(&router, &path, "GET", serde_json::Value::Null, "", "", "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    let (cookie, _) = login(&router, "admin", "correct horse battery").await;
    let response = request(
        &router,
        &path,
        "GET",
        serde_json::Value::Null,
        &cookie,
        "",
        "",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        to_bytes(response.into_body(), 4 * 1024 * 1024)
            .await
            .unwrap()
            .as_ref(),
        bytes
    );
    let response = request(&router, "/b64", "GET", serde_json::Value::Null, "", "", "").await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let response = request(
        &router,
        "/api/v1/overview",
        "GET",
        serde_json::Value::Null,
        &cookie,
        "",
        "",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = String::from_utf8(
        to_bytes(response.into_body(), 4 * 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(!body.contains("unknownFutureField"));
    assert!(!body.contains("11111111-1111-4111"));
}

async fn token(router: &axum::Router, cookie: &str, csrf: &str) -> xrat_web::types::CreatedToken {
    let response = request(
        router,
        "/api/v1/tokens",
        "POST",
        serde_json::json!({"name":"subscription", "scope":"read"}),
        cookie,
        csrf,
        "http://127.0.0.1:18203",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap()
}
async fn decoded(router: &axum::Router, path: &str) -> String {
    let response = request(router, path, "GET", serde_json::Value::Null, "", "", "").await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    String::from_utf8(
        STANDARD
            .decode(to_bytes(response.into_body(), 1_048_576).await.unwrap())
            .unwrap(),
    )
    .unwrap()
}

#[tokio::test]
async fn subscription_query_tokens_obey_revocation_roles_and_credential_versions_without_legacy_fallback()
 {
    let (router, state) = fixture().await;
    tables::seed(&state).await;
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let token = token(&router, &cookie, &me.csrf_token).await;
    let path = format!("/b64?token={}&enabled=false&state=all", token.token);
    assert_eq!(decoded(&router, &path).await.lines().count(), 2);
    // A missing token may use legacy public export; an invalid supplied token may never do so.
    state
        .db
        .web_delete_token(&super::super::security::digest(&token.token), "admin")
        .await
        .unwrap();
    assert_eq!(
        request(&router, &path, "GET", serde_json::Value::Null, "", "", "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    state
        .db
        .web_save_user(&xrat_db::WebUser {
            username: "share-user".into(),
            role: "operator".into(),
            password_hash: hash_password("test password long").unwrap(),
            credential_version: "v1".into(),
            enabled: true,
            must_change_password: false,
        })
        .await
        .unwrap();
    let (cookie, me) = login(&router, "share-user", "test password long").await;
    let token = self::token(&router, &cookie, &me.csrf_token).await;
    let path = format!("/b64?token={}", token.token);
    assert_eq!(decoded(&router, &path).await.lines().count(), 2);
    let mut record = super::super::security::user(&state, "share-user")
        .await
        .unwrap()
        .unwrap();
    record.role = "viewer".into();
    state.db.web_save_user(&record).await.unwrap();
    assert_eq!(
        request(&router, &path, "GET", serde_json::Value::Null, "", "", "")
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    record.role = "operator".into();
    record.must_change_password = true;
    state.db.web_save_user(&record).await.unwrap();
    assert_eq!(
        request(&router, &path, "GET", serde_json::Value::Null, "", "", "")
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    record.must_change_password = false;
    record.enabled = false;
    state.db.web_save_user(&record).await.unwrap();
    assert_eq!(
        request(&router, &path, "GET", serde_json::Value::Null, "", "", "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    record.enabled = true;
    record.credential_version = "v2".into();
    state.db.web_save_user(&record).await.unwrap();
    assert_eq!(
        request(&router, &path, "GET", serde_json::Value::Null, "", "", "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn share_exports_match_subscription_filters_and_selected_ids_including_deleted() {
    let (router, state) = fixture().await;
    let ids = tables::seed(&state).await;
    state.db.delete_config(ids[0].into()).await.unwrap();
    state
        .db
        .set_config_enabled(ids[1].into(), false)
        .await
        .unwrap();
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let token = token(&router, &cookie, &me.csrf_token).await;
    let filter = ConfigFilter {
        show_deleted: true,
        sort: "name".into(),
        order: "desc".into(),
        ..Default::default()
    };
    let response = request(
        &router,
        "/api/v1/configs/share",
        "POST",
        serde_json::to_value(ConfigShareRequest::Filtered {
            filter: filter.clone(),
        })
        .unwrap(),
        &cookie,
        &me.csrf_token,
        "http://127.0.0.1:18203",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let exported: ConfigShareResponse =
        serde_json::from_slice(&to_bytes(response.into_body(), 1_048_576).await.unwrap()).unwrap();
    assert_eq!(exported.count, 2);
    assert!(exported.raw.lines().next().unwrap().contains("two.example"));
    let path = format!(
        "/b64?token={}&enabled=false&show_deleted=true&sort=name&order=desc",
        token.token
    );
    assert_eq!(decoded(&router, &path).await, exported.raw);
    let path = format!("/b64?token={}&enabled=false&state=disabled", token.token);
    let disabled = decoded(&router, &path).await;
    assert_eq!(disabled.lines().count(), 1);
    assert!(disabled.contains("two.example"));
    let path = format!(
        "/b64?token={}&enabled=false&state=deleted&show_deleted=true",
        token.token
    );
    assert!(decoded(&router, &path).await.contains("one.example"));
    let response = request(
        &router,
        "/api/v1/configs/share",
        "POST",
        serde_json::json!({"scope":"selected", "ids": [ids[1], ids[0]]}),
        &cookie,
        &me.csrf_token,
        "http://127.0.0.1:18203",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let selected: ConfigShareResponse =
        serde_json::from_slice(&to_bytes(response.into_body(), 1_048_576).await.unwrap()).unwrap();
    assert_eq!(selected.raw, exported.raw);
    for bad in [
        serde_json::json!({"scope":"selected","ids":[]}),
        serde_json::json!({"scope":"selected","ids":[ids[0],ids[0]]}),
        serde_json::json!({"scope":"filtered","filter":{"test":"typo"}}),
    ] {
        assert_eq!(
            request(
                &router,
                "/api/v1/configs/share",
                "POST",
                bad,
                &cookie,
                &me.csrf_token,
                "http://127.0.0.1:18203"
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
    }
    assert_eq!(
        request(
            &router,
            "/api/v1/configs/share",
            "POST",
            serde_json::json!({"scope":"selected","ids":[ids[0]]}),
            &cookie,
            "",
            "http://127.0.0.1:18203"
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    state.db.restore_config(ids[0].into()).await.unwrap();
    let path = format!(
        "/b64?token={}&enabled=false&state=deleted&show_deleted=true",
        token.token
    );
    assert!(decoded(&router, &path).await.is_empty());
}

#[tokio::test]
async fn subscription_accepts_legacy_keys_and_account_tokens_with_token_precedence() {
    let (_, mut state) = fixture().await;
    state.api_key = Some("legacy-secret".into());
    let router = crate::server::build_router(state.clone());
    tables::seed(&state).await;
    assert_eq!(
        request(&router, "/b64", "GET", serde_json::Value::Null, "", "", "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        decoded(&router, "/b64?key=legacy-secret&enabled=false&state=all")
            .await
            .lines()
            .count(),
        2
    );
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let token = token(&router, &cookie, &me.csrf_token).await;
    let url = format!(
        "/b64?token={}&key=wrong&enabled=false&state=all",
        token.token
    );
    assert_eq!(decoded(&router, &url).await.lines().count(), 2);
    for url in [
        "/b64?token=wrong&key=legacy-secret",
        "/b64?token=&key=legacy-secret",
    ] {
        assert_eq!(
            request(&router, url, "GET", serde_json::Value::Null, "", "", "")
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
}

#[tokio::test]
async fn selected_read_token_mints_subscription_only_credentials_and_revocation_cascades() {
    let (router, state) = fixture().await;
    tables::seed(&state).await;
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    token(&router, &cookie, &me.csrf_token).await;
    let parent = state.db.web_tokens("admin").await.unwrap().remove(0);
    let endpoint = format!("/api/v1/tokens/{}/subscription", parent.token_hash);
    assert_eq!(
        request(
            &router,
            &endpoint,
            "POST",
            serde_json::Value::Null,
            &cookie,
            "",
            "http://127.0.0.1:18203"
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let mut secrets = Vec::new();
    for _ in 0..2 {
        let response = request(
            &router,
            &endpoint,
            "POST",
            serde_json::Value::Null,
            &cookie,
            &me.csrf_token,
            "http://127.0.0.1:18203",
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let created: xrat_web::types::CreatedToken =
            serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
        assert!(created.token.starts_with("sub_"));
        assert_eq!(
            decoded(&router, &format!("/b64?token={}", created.token))
                .await
                .lines()
                .count(),
            2
        );
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/overview")
                    .header("authorization", format!("Bearer {}", created.token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        secrets.push(created.token);
    }
    assert_ne!(secrets[0], secrets[1]);
    let mut other = parent.clone();
    other.token_hash = "other-owner-token".into();
    other.username = "someone-else".into();
    state.db.web_save_token(&other).await.unwrap();
    other.token_hash = "manage-token".into();
    other.username = "admin".into();
    other.scope = "manage".into();
    state.db.web_save_token(&other).await.unwrap();
    for id in ["other-owner-token", "manage-token", "not-a-token"] {
        assert_eq!(
            request(
                &router,
                &format!("/api/v1/tokens/{id}/subscription"),
                "POST",
                serde_json::Value::Null,
                &cookie,
                &me.csrf_token,
                "http://127.0.0.1:18203"
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
    }
    state
        .db
        .web_delete_token(&parent.token_hash, "admin")
        .await
        .unwrap();
    for secret in secrets {
        assert_eq!(
            request(
                &router,
                &format!("/b64?token={secret}"),
                "GET",
                serde_json::Value::Null,
                "",
                "",
                ""
            )
            .await
            .status(),
            StatusCode::UNAUTHORIZED
        );
    }
}

#[tokio::test]
async fn qr_preview_reports_capacity_without_exposing_raw_configs() {
    let (router, state) = fixture().await;
    let ids = tables::seed(&state).await;
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let response = request(
        &router,
        "/api/v1/configs/share/preview",
        "POST",
        serde_json::json!({"scope":"selected", "ids":[ids[0]]}),
        &cookie,
        &me.csrf_token,
        "http://127.0.0.1:18203",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 65536).await.unwrap();
    let preview: xrat_web::configs::sharing::ConfigSharePreview =
        serde_json::from_slice(&bytes).unwrap();
    assert_eq!(preview.count, 1);
    assert!(preview.qr_supported);
    assert!(!String::from_utf8_lossy(&bytes).contains("vless://"));
    let raw = (0..30).map(|i| format!("vless://00000000-0000-0000-0000-{i:012}@node-{i}.example:443?security=tls#LongConfig{}", "x".repeat(100))).collect::<Vec<_>>().join("\n");
    state
        .db
        .import_nodes(
            &xrat_db::ImportSource {
                kind: xrat_db::SourceKind::RawText,
                value: "qr-capacity-fixture".into(),
                name: None,
            },
            &xrat_config::parse_text(&raw),
        )
        .await
        .unwrap();
    let response = request(
        &router,
        "/api/v1/configs/share/preview",
        "POST",
        serde_json::json!({"scope":"filtered", "filter":{}}),
        &cookie,
        &me.csrf_token,
        "http://127.0.0.1:18203",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let preview: xrat_web::configs::sharing::ConfigSharePreview =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    assert!(!preview.qr_supported);
    assert!(preview.count > 30);
}

#[tokio::test]
async fn subscription_source_sharing_is_on_demand_and_requires_operator_access() {
    let (router, state) = fixture().await;
    tables::seed(&state).await;
    let nodes = xrat_config::parse_text(
        "vless://33333333-3333-3333-3333-333333333333@source.example:443#Source",
    );
    let source_url = "https://source.example/subscription?token=dummy-private-token";
    state
        .db
        .import_nodes(
            &xrat_db::ImportSource {
                kind: xrat_db::SourceKind::Url,
                value: source_url.into(),
                name: Some("URL source".into()),
            },
            &nodes,
        )
        .await
        .unwrap();
    let subs = state.db.list_subscriptions().await.unwrap();
    let url_sub = subs.iter().find(|row| row.source_kind == "url").unwrap();
    let text_sub = subs
        .iter()
        .find(|row| row.source_kind == "raw_text")
        .unwrap();
    let path = format!("/api/v1/subscriptions/{}/source", url_sub.id.0);
    assert_eq!(
        request(&router, &path, "GET", serde_json::Value::Null, "", "", "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    let (cookie, _) = login(&router, "admin", "correct horse battery").await;
    let response = request(
        &router,
        &path,
        "GET",
        serde_json::Value::Null,
        &cookie,
        "",
        "",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let source: xrat_web::types::SubscriptionSource =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    assert_eq!(source.url, source_url);
    let preview = request(
        &router,
        &format!("{path}/preview"),
        "GET",
        serde_json::Value::Null,
        &cookie,
        "",
        "",
    )
    .await;
    let preview = to_bytes(preview.into_body(), 65536).await.unwrap();
    assert!(
        serde_json::from_slice::<xrat_web::types::SubscriptionSourcePreview>(&preview)
            .unwrap()
            .qr_supported
    );
    assert!(!String::from_utf8_lossy(&preview).contains("dummy-private-token"));
    assert_eq!(
        request(
            &router,
            &format!("/api/v1/subscriptions/{}/source", text_sub.id.0),
            "GET",
            serde_json::Value::Null,
            &cookie,
            "",
            ""
        )
        .await
        .status(),
        StatusCode::BAD_REQUEST
    );
    let overview = request(
        &router,
        "/api/v1/overview",
        "GET",
        serde_json::Value::Null,
        &cookie,
        "",
        "",
    )
    .await;
    assert!(
        !String::from_utf8_lossy(&to_bytes(overview.into_body(), 1_048_576).await.unwrap())
            .contains("dummy-private-token")
    );
    state
        .db
        .web_save_user(&xrat_db::WebUser {
            username: "source-viewer".into(),
            role: "viewer".into(),
            password_hash: hash_password("viewer password long").unwrap(),
            credential_version: "v1".into(),
            enabled: true,
            must_change_password: false,
        })
        .await
        .unwrap();
    let (viewer, _) = login(&router, "source-viewer", "viewer password long").await;
    assert_eq!(
        request(
            &router,
            &path,
            "GET",
            serde_json::Value::Null,
            &viewer,
            "",
            ""
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn multiselect_exports_and_subscription_links_use_identical_matching() {
    let (router, state) = fixture().await;
    let ids = tables::seed(&state).await;
    state
        .db
        .set_config_enabled(ids[1].into(), false)
        .await
        .unwrap();
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let token = token(&router, &cookie, &me.csrf_token).await;
    let filter = ConfigFilter {
        protocol: "vless,ss".into(),
        state: "enabled,disabled".into(),
        test: "untested,passed".into(),
        ..Default::default()
    };
    let response = request(
        &router,
        "/api/v1/configs/share",
        "POST",
        serde_json::to_value(ConfigShareRequest::Filtered { filter }).unwrap(),
        &cookie,
        &me.csrf_token,
        "http://127.0.0.1:18203",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let shared: ConfigShareResponse =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    assert_eq!(shared.count, 2);
    let raw = decoded(&router, &format!("/b64?token={}&enabled=false&protocol=vless,ss&state=enabled,disabled&test=untested,passed", token.token)).await;
    assert_eq!(raw, shared.raw);
}
