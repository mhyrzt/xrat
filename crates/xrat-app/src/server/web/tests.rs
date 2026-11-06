use super::*;
use crate::app::{
    config::{SecretString, ServerSettings},
    tests::TestAppBuilder,
};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use xrat_web::types::{LoginResponse, Role, SaveUserRequest};

async fn fixture() -> (axum::Router, ServerState) {
    let context = TestAppBuilder::new("web-auth").build().await;
    let mut settings = ServerSettings::default();
    settings.web.enabled = true;
    settings.web.admin_username = "admin".into();
    settings.web.admin_password_hash = Some(SecretString::Literal(
        hash_password("correct horse battery").unwrap(),
    ));
    let mut state =
        ServerState::from_settings(context.db.clone(), &settings, &context.app_config.routing)
            .unwrap();
    state.services = context.services();
    std::sync::Arc::get_mut(state.web.as_mut().unwrap())
        .unwrap()
        .context = Some(context);
    (crate::server::build_router(state.clone()), state)
}
async fn request(
    router: &axum::Router,
    path: &str,
    method: &str,
    body: serde_json::Value,
    cookie: &str,
    csrf: &str,
    origin: &str,
) -> axum::response::Response {
    router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("content-type", "application/json")
                .header("origin", origin)
                .header("cookie", cookie)
                .header("x-csrf-token", csrf)
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}
async fn login(router: &axum::Router, username: &str, password: &str) -> (String, LoginResponse) {
    let response = request(
        router,
        "/api/v1/login",
        "POST",
        serde_json::json!({"username":username,"password":password}),
        "",
        "",
        "http://127.0.0.1:18203",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let cookie = response.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    let body = to_bytes(response.into_body(), 65536).await.unwrap();
    (cookie, serde_json::from_slice(&body).unwrap())
}
#[tokio::test]
async fn login_csrf_admin_protection_and_logout() {
    let (router, state) = fixture().await;
    assert_eq!(
        request(
            &router,
            "/api/v1/users",
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
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    assert!(me.user.configured);
    let user = serde_json::to_value(SaveUserRequest {
        username: "viewer".into(),
        role: Role::Viewer,
        enabled: true,
        password: Some("temporary password".into()),
    })
    .unwrap();
    assert_eq!(
        request(
            &router,
            "/api/v1/users",
            "POST",
            user.clone(),
            &cookie,
            "",
            "http://127.0.0.1:18203"
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &router,
            "/api/v1/users",
            "POST",
            user.clone(),
            &cookie,
            &me.csrf_token,
            "https://attacker.example"
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &router,
            "/api/v1/users",
            "POST",
            user,
            &cookie,
            &me.csrf_token,
            "http://127.0.0.1:18203"
        )
        .await
        .status(),
        StatusCode::OK
    );
    let record = state.db.web_users().await.unwrap().remove(0);
    assert_ne!(record.password_hash, "temporary password");
    assert!(record.must_change_password);
    assert_eq!(request(&router, "/api/v1/users", "POST", serde_json::json!({"username":"admin","role":"viewer","enabled":false,"password":"temporary password"}), &cookie, &me.csrf_token, "http://127.0.0.1:18203").await.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        request(
            &router,
            "/api/v1/logout",
            "POST",
            serde_json::Value::Null,
            &cookie,
            &me.csrf_token,
            "http://127.0.0.1:18203"
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        request(
            &router,
            "/api/v1/me",
            "GET",
            serde_json::Value::Null,
            &cookie,
            "",
            ""
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
}
#[tokio::test]
async fn temporary_password_can_be_changed_and_viewer_cannot_mutate() {
    let (router, state) = fixture().await;
    state
        .db
        .web_save_user(&xrat_db::WebUser {
            username: "viewer".into(),
            role: "viewer".into(),
            password_hash: hash_password("temporary password").unwrap(),
            credential_version: "v1".into(),
            enabled: true,
            must_change_password: true,
        })
        .await
        .unwrap();
    let (cookie, me) = login(&router, "viewer", "temporary password").await;
    assert_eq!(
        request(
            &router,
            "/api/v1/me",
            "GET",
            serde_json::Value::Null,
            &cookie,
            "",
            ""
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        request(
            &router,
            "/api/v1/overview",
            "GET",
            serde_json::Value::Null,
            &cookie,
            "",
            ""
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(request(&router, "/api/v1/password", "POST", serde_json::json!({"current_password":"temporary password","new_password":"replacement password"}), &cookie, &me.csrf_token, "http://127.0.0.1:18203").await.status(), StatusCode::OK);
    assert_eq!(
        request(
            &router,
            "/api/v1/me",
            "GET",
            serde_json::Value::Null,
            &cookie,
            "",
            ""
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    let (cookie, me) = login(&router, "viewer", "replacement password").await;
    assert!(!me.user.must_change_password);
    for path in ["/api/v1/actions", "/api/v1/users", "/api/v1/tokens"] {
        assert_eq!(
            request(
                &router,
                path,
                "POST",
                serde_json::json!({"action":"disconnect"}),
                &cookie,
                &me.csrf_token,
                "http://127.0.0.1:18203"
            )
            .await
            .status(),
            StatusCode::FORBIDDEN
        );
    }
}
#[test]
fn transport_rejects_insecure_public_binds_and_proxy_without_origin() {
    let mut settings = ServerSettings::default();
    settings.web.enabled = true;
    assert!(settings.web.origin("0.0.0.0", 18203).is_err());
    settings.web.transport = crate::app::config::WebTransport::ProxyHttps;
    assert!(settings.web.origin("0.0.0.0", 18203).is_err());
    settings.web.public_url = Some("https://xrat.example".into());
    assert_eq!(
        settings.web.origin("0.0.0.0", 18203).unwrap(),
        "https://xrat.example"
    );
}

#[tokio::test]
async fn scoped_tokens_and_expired_sessions_are_rejected() {
    let (router, state) = fixture().await;
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let response = request(
        &router,
        "/api/v1/tokens",
        "POST",
        serde_json::json!({"name":"automation","scope":"read"}),
        &cookie,
        &me.csrf_token,
        "http://127.0.0.1:18203",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 65536).await.unwrap();
    let token: xrat_web::types::CreatedToken = serde_json::from_slice(&body).unwrap();
    for (method, path, expected) in [
        ("GET", "/api/v1/overview", StatusCode::OK),
        ("POST", "/api/v1/actions", StatusCode::FORBIDDEN),
    ] {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .header("authorization", format!("Bearer {}", token.token))
                    .header("content-type", "application/json")
                    .body(Body::from("{\"action\":\"disconnect\"}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
    }
    let hash = super::security::digest(&token.token);
    state.db.web_delete_token(&hash, "admin").await.unwrap();
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/overview")
                .header("authorization", format!("Bearer {}", token.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let session_token = cookie.strip_prefix("xrat_session=").unwrap();
    let session_hash = super::security::digest(session_token);
    state.db.web_touch_session(&session_hash, 0).await.unwrap();
    assert_eq!(
        request(
            &router,
            "/api/v1/me",
            "GET",
            serde_json::Value::Null,
            &cookie,
            "",
            ""
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn proxy_https_uses_secure_cookies_without_certificate_files() {
    let (_, mut state) = fixture().await;
    let context = state.web.as_ref().unwrap().context.clone().unwrap();
    let mut settings = ServerSettings {
        host: "0.0.0.0".into(),
        ..Default::default()
    };
    settings.web.enabled = true;
    settings.web.admin_username = "admin".into();
    settings.web.admin_password_hash = Some(SecretString::Literal(
        hash_password("correct horse battery").unwrap(),
    ));
    settings.web.transport = crate::app::config::WebTransport::ProxyHttps;
    settings.web.public_url = Some("https://public.example".into());
    state = ServerState::from_settings(context.db.clone(), &settings, &context.app_config.routing)
        .unwrap();
    let router = crate::server::build_router(state);
    let response = request(
        &router,
        "/api/v1/login",
        "POST",
        serde_json::json!({"username":"admin","password":"correct horse battery"}),
        "",
        "",
        "https://public.example",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .contains("; Secure")
    );
    assert_eq!(
        response.headers()["strict-transport-security"],
        "max-age=31536000"
    );
    assert!(settings.web.certificate.is_none());
    let response = request(
        &router,
        "/api/v1/login",
        "POST",
        serde_json::json!({"username":"admin","password":"correct horse battery"}),
        "",
        "",
        "http://public.example",
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn internal_errors_do_not_disclose_database_details() {
    use axum::response::IntoResponse;
    let response = crate::server::ServerError::Database(xrat_db::DbError::Sqlx(
        sqlx::Error::Protocol("private database address and query".into()),
    ))
    .into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = to_bytes(response.into_body(), 65536).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&body).unwrap()["error"],
        "internal server error"
    );
}

#[tokio::test]
async fn engine_logs_require_operator_and_handle_no_session() {
    let (router, _) = fixture().await;
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    assert_eq!(
        request(
            &router,
            "/api/v1/logs/engine",
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
    let response = request(
        &router,
        "/api/v1/logs/engine",
        "GET",
        serde_json::Value::Null,
        &cookie,
        "",
        "",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        &to_bytes(response.into_body(), 1024).await.unwrap()[..],
        b"[]"
    );
    let response = request(&router, "/api/v1/users", "POST", serde_json::json!({"username":"logviewer", "role":"viewer", "enabled":true, "password":"temporary password"}), &cookie, &me.csrf_token, "http://127.0.0.1:18203").await;
    assert_eq!(response.status(), StatusCode::OK);
    let (viewer, _) = login(&router, "logviewer", "temporary password").await;
    assert_eq!(
        request(
            &router,
            "/api/v1/logs/engine",
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
async fn host_unlock_requires_password_and_is_scoped_expiring_and_revocable() {
    use super::security::{Principal, digest, now};
    use axum::http::HeaderMap;
    let (router, state) = fixture().await;
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let origin = "http://127.0.0.1:18203";
    let principal = Principal {
        user: me.user.clone(),
        version: state.web.as_ref().unwrap().admin.credential_version.clone(),
        session_hash: Some(digest(cookie.strip_prefix("xrat_session=").unwrap())),
        csrf_hash: None,
        read_only_token: false,
        token_hash: None,
    };
    assert!(super::host::require_unlocked(&state, &principal, &HeaderMap::new()).is_err());
    assert_eq!(
        request(
            &router,
            "/api/v1/actions",
            "POST",
            serde_json::json!({"action":"rotation", "enabled":false}),
            &cookie,
            &me.csrf_token,
            origin
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &router,
            "/api/v1/host/unlock",
            "POST",
            serde_json::json!({"password":"wrong password"}),
            &cookie,
            &me.csrf_token,
            origin
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        request(
            &router,
            "/api/v1/host/unlock",
            "POST",
            serde_json::json!({"password":"correct horse battery"}),
            &cookie,
            "",
            origin
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    let response = request(
        &router,
        "/api/v1/host/unlock",
        "POST",
        serde_json::json!({"password":"correct horse battery"}),
        &cookie,
        &me.csrf_token,
        origin,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let grant: xrat_web::types::HostUnlock =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    assert!((299..=300).contains(&(grant.expires_at - now())));
    let mut headers = HeaderMap::new();
    headers.insert("x-xrat-host-unlock", grant.token.parse().unwrap());
    assert!(super::host::require_unlocked(&state, &principal, &headers).is_ok());
    let mut other = principal.clone();
    other.session_hash = Some("other-session".into());
    assert!(super::host::require_unlocked(&state, &other, &headers).is_err());
    other = principal.clone();
    other.user.role = Role::Operator;
    assert!(super::host::require_unlocked(&state, &other, &headers).is_err());
    assert_eq!(
        request(
            &router,
            "/api/v1/host/lock",
            "POST",
            serde_json::Value::Null,
            &cookie,
            &me.csrf_token,
            origin
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert!(super::host::require_unlocked(&state, &principal, &headers).is_err());
    state
        .web
        .as_ref()
        .unwrap()
        .host_unlocks
        .lock()
        .unwrap()
        .insert(
            digest(&grant.token),
            (principal.session_hash.clone().unwrap(), now() - 1),
        );
    assert!(super::host::require_unlocked(&state, &principal, &headers).is_err());
    for action in [
        xrat_web::types::Action::Rotation { enabled: true },
        xrat_web::types::Action::Purge,
        xrat_web::types::Action::ClearEvents,
        xrat_web::types::Action::Validate,
        xrat_web::types::Action::Tun { enabled: true },
        xrat_web::types::Action::GeoIpUpdate,
        xrat_web::types::Action::Upgrade { version: None },
        xrat_web::types::Action::InstallEngine {
            engine: xrat_web::types::Engine::Xray,
        },
    ] {
        assert!(super::host::protected(&action));
    }
}

#[tokio::test]
async fn settings_editor_preserves_validation_and_revision_checks() {
    let (router, state) = fixture().await;
    let config = &state
        .web
        .as_ref()
        .unwrap()
        .context
        .as_ref()
        .unwrap()
        .runtime_paths
        .config_path;
    std::fs::create_dir_all(config.parent().unwrap()).unwrap();
    std::fs::write(config, "[runtime]\nreplace_active_session = false\n").unwrap();
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let response = request(
        &router,
        "/api/v1/settings",
        "GET",
        serde_json::Value::Null,
        &cookie,
        "",
        "",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let initial: xrat_web::types::Settings =
        serde_json::from_slice(&to_bytes(response.into_body(), 1_048_576).await.unwrap()).unwrap();
    let path = "runtime.replace_active_session";
    assert_eq!(
        initial
            .items
            .iter()
            .find(|setting| setting.path == path)
            .unwrap()
            .kind,
        "bool"
    );
    let origin = "http://127.0.0.1:18203";
    let update = |revision: &str, value: &str| serde_json::json!({"revision": revision, "path": path, "value": value});
    assert_eq!(
        request(
            &router,
            "/api/v1/settings",
            "POST",
            update(&initial.revision, "true"),
            &cookie,
            &me.csrf_token,
            origin
        )
        .await
        .status(),
        StatusCode::OK
    );
    assert_eq!(
        request(
            &router,
            "/api/v1/settings",
            "POST",
            update(&initial.revision, "false"),
            &cookie,
            &me.csrf_token,
            origin
        )
        .await
        .status(),
        StatusCode::BAD_REQUEST
    );
    let response = request(
        &router,
        "/api/v1/settings",
        "GET",
        serde_json::Value::Null,
        &cookie,
        "",
        "",
    )
    .await;
    let current: xrat_web::types::Settings =
        serde_json::from_slice(&to_bytes(response.into_body(), 1_048_576).await.unwrap()).unwrap();
    assert_eq!(
        current
            .items
            .iter()
            .find(|setting| setting.path == path)
            .unwrap()
            .value,
        "true"
    );
    let response = request(
        &router,
        "/api/v1/settings",
        "POST",
        update(&current.revision, "invalid-bool"),
        &cookie,
        &me.csrf_token,
        origin,
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), 65536).await.unwrap();
    assert!(String::from_utf8_lossy(&body).contains("Expected true or false"));
}

#[tokio::test]
async fn settings_batch_is_atomic_and_checks_revision() {
    let (router, state) = fixture().await;
    let config = &state
        .web
        .as_ref()
        .unwrap()
        .context
        .as_ref()
        .unwrap()
        .runtime_paths
        .config_path;
    std::fs::create_dir_all(config.parent().unwrap()).unwrap();
    let original = "[runtime]\nreplace_active_session = false\n";
    std::fs::write(config, original).unwrap();
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let response = request(
        &router,
        "/api/v1/settings",
        "GET",
        serde_json::Value::Null,
        &cookie,
        "",
        "",
    )
    .await;
    let initial: xrat_web::types::Settings =
        serde_json::from_slice(&to_bytes(response.into_body(), 1_048_576).await.unwrap()).unwrap();
    let first = "runtime.replace_active_session";
    let second = initial
        .items
        .iter()
        .find(|setting| setting.kind == "bool" && setting.path != first)
        .unwrap();
    let batch = |second_value: &str| {
        serde_json::json!({"revision": initial.revision, "changes": [
            {"path": first, "value": "true"}, {"path": second.path, "value": second_value}
        ]})
    };
    let origin = "http://127.0.0.1:18203";
    assert_eq!(
        request(
            &router,
            "/api/v1/settings/batch",
            "POST",
            batch("invalid-bool"),
            &cookie,
            &me.csrf_token,
            origin
        )
        .await
        .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(std::fs::read_to_string(config).unwrap(), original);
    let restricted = serde_json::json!({"revision": initial.revision, "changes": [
        {"path": first, "value": "true"}, {"path": "server.web.enabled", "value": "false"}
    ]});
    assert_eq!(
        request(
            &router,
            "/api/v1/settings/batch",
            "POST",
            restricted,
            &cookie,
            &me.csrf_token,
            origin
        )
        .await
        .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(std::fs::read_to_string(config).unwrap(), original);
    let value = if second.value == "true" {
        "false"
    } else {
        "true"
    };
    assert_eq!(
        request(
            &router,
            "/api/v1/settings/batch",
            "POST",
            batch(value),
            &cookie,
            &me.csrf_token,
            origin
        )
        .await
        .status(),
        StatusCode::OK
    );
    let saved = std::fs::read_to_string(config).unwrap();
    let response = request(
        &router,
        "/api/v1/settings",
        "GET",
        serde_json::Value::Null,
        &cookie,
        "",
        "",
    )
    .await;
    let current: xrat_web::types::Settings =
        serde_json::from_slice(&to_bytes(response.into_body(), 1_048_576).await.unwrap()).unwrap();
    assert_eq!(
        current
            .items
            .iter()
            .find(|setting| setting.path == first)
            .unwrap()
            .value,
        "true"
    );
    assert_eq!(
        current
            .items
            .iter()
            .find(|setting| setting.path == second.path)
            .unwrap()
            .value,
        value
    );
    assert_eq!(
        request(
            &router,
            "/api/v1/settings/batch",
            "POST",
            batch(value),
            &cookie,
            &me.csrf_token,
            origin
        )
        .await
        .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(std::fs::read_to_string(config).unwrap(), saved);
}

#[tokio::test]
async fn account_edit_and_password_reset_preserve_unrelated_fields() {
    let (router, state) = fixture().await;
    let (cookie, me) = login(&router, "admin", "correct horse battery").await;
    let origin = "http://127.0.0.1:18203";
    let save = |role, enabled, password: Option<&str>| {
        serde_json::to_value(SaveUserRequest {
            username: "account-editor-test".into(),
            role,
            enabled,
            password: password.map(str::to_string),
        })
        .unwrap()
    };
    assert_eq!(
        request(
            &router,
            "/api/v1/users",
            "POST",
            save(Role::Viewer, true, Some("temporary password")),
            &cookie,
            &me.csrf_token,
            origin
        )
        .await
        .status(),
        StatusCode::OK
    );
    let (old_session, _) = login(&router, "account-editor-test", "temporary password").await;
    assert_eq!(
        request(
            &router,
            "/api/v1/users",
            "POST",
            save(Role::Operator, false, None),
            &cookie,
            &me.csrf_token,
            origin
        )
        .await
        .status(),
        StatusCode::OK
    );
    let record = super::security::user(&state, "account-editor-test")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(record.role, "operator");
    assert!(!record.enabled);
    assert!(record.must_change_password);
    assert!(
        super::security::verify(
            state.web.as_ref().unwrap(),
            "temporary password".into(),
            record.password_hash
        )
        .await
        .unwrap()
    );
    assert_eq!(
        request(
            &router,
            "/api/v1/me",
            "GET",
            serde_json::Value::Null,
            &old_session,
            "",
            ""
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(
            &router,
            "/api/v1/users",
            "POST",
            save(Role::Operator, false, Some("replacement password")),
            &cookie,
            &me.csrf_token,
            origin
        )
        .await
        .status(),
        StatusCode::OK
    );
    let record = super::security::user(&state, "account-editor-test")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(record.role, "operator");
    assert!(!record.enabled);
    assert!(record.must_change_password);
    assert!(
        !super::security::verify(
            state.web.as_ref().unwrap(),
            "temporary password".into(),
            record.password_hash.clone()
        )
        .await
        .unwrap()
    );
    assert!(
        super::security::verify(
            state.web.as_ref().unwrap(),
            "replacement password".into(),
            record.password_hash
        )
        .await
        .unwrap()
    );
}

mod tables;

mod dashboard;

mod sharing;

mod bootstrap;
mod imports;
