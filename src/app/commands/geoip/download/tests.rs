use std::time::Duration;

use axum::http::StatusCode;

use super::super::edition::{MmdbEdition, SUPPORTED_EDITIONS};
use super::executor::{
    DownloadOutcome, build_download_url, download_one, download_one_with_client,
};
use super::progress::ensure_non_empty_download;
use super::request::{DownloadRequest, resolve_requested_editions};
use super::summary::{DownloadFailure, DownloadSummary};
use crate::app::AppError;

fn test_request(mmdb_dir: std::path::PathBuf) -> DownloadRequest {
    DownloadRequest {
        editions: vec![MmdbEdition::Country],
        mmdb_dir,
        force: false,
        url_template: "https://example.com/{edition}.mmdb".to_string(),
        timeout_secs: 1,
        quiet: true,
    }
}

#[test]
fn builds_download_url_from_template() {
    assert_eq!(
        build_download_url("https://example.com/{edition}.mmdb", MmdbEdition::Asn),
        "https://example.com/GeoLite2-ASN.mmdb"
    );
}

#[test]
fn defaults_download_to_country() {
    assert_eq!(
        resolve_requested_editions(&[], false).unwrap(),
        vec![MmdbEdition::Country]
    );
}

#[test]
fn resolves_all_download_editions() {
    assert_eq!(
        resolve_requested_editions(&[], true).unwrap(),
        SUPPORTED_EDITIONS.to_vec()
    );
}

#[test]
fn formats_summary_line() {
    let summary = DownloadSummary {
        downloaded: 2,
        skipped: 1,
        failed: vec![DownloadFailure {
            edition: MmdbEdition::Asn,
            reason: "HTTP 500".to_string(),
        }],
    };

    assert_eq!(summary.format(), "summary: downloaded=2 skipped=1 failed=1");
}

#[test]
fn failure_summary_includes_reason() {
    let summary = DownloadSummary {
        downloaded: 0,
        skipped: 0,
        failed: vec![DownloadFailure {
            edition: MmdbEdition::City,
            reason: "HTTP 404 Not Found".to_string(),
        }],
    };

    assert_eq!(summary.format(), "summary: downloaded=0 skipped=0 failed=1");
}

#[test]
fn rejects_empty_downloads() {
    let error = ensure_non_empty_download(
        0,
        &None,
        MmdbEdition::City,
        "https://example.com/GeoLite2-City.mmdb",
    )
    .expect_err("empty download should fail");

    match error {
        AppError::GeoipDownload {
            edition,
            url,
            reason,
        } => {
            assert_eq!(edition, "GeoLite2-City");
            assert_eq!(url, "https://example.com/GeoLite2-City.mmdb");
            assert!(reason.contains("empty file"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn skips_existing_destination_without_force() {
    let root = std::env::temp_dir().join("xrat-geoip-download-skip-test");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("root should be created");
    std::fs::write(root.join("GeoLite2-Country.mmdb"), [1_u8; 4])
        .expect("fixture file should exist");

    let outcome = super::executor::download_one(&test_request(root), MmdbEdition::Country)
        .await
        .expect("existing file should skip");

    assert!(matches!(outcome, DownloadOutcome::Skipped));
}

#[tokio::test]
async fn downloads_file_from_stubbed_http_server() {
    let root = std::env::temp_dir().join("xrat-geoip-download-http-success-test");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("root should be created");

    let server = spawn_download_server(StatusCode::OK, vec![1_u8, 2, 3, 4]).await;
    let request = DownloadRequest {
        url_template: format!("{server}/{{edition}}.mmdb"),
        mmdb_dir: root.clone(),
        ..test_request(root.clone())
    };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .unwrap();

    let outcome = download_one_with_client(&client, &request, MmdbEdition::Country)
        .await
        .expect("download should succeed");

    assert!(matches!(outcome, DownloadOutcome::Downloaded));
    assert_eq!(
        std::fs::read(root.join("GeoLite2-Country.mmdb")).expect("file should exist"),
        vec![1_u8, 2, 3, 4]
    );
}

#[tokio::test]
async fn reports_http_error_with_url() {
    let root = std::env::temp_dir().join("xrat-geoip-download-http-fail-test");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("root should be created");

    let server = spawn_download_server(StatusCode::NOT_FOUND, b"missing".to_vec()).await;
    let request = DownloadRequest {
        url_template: format!("{server}/{{edition}}.mmdb"),
        mmdb_dir: root.clone(),
        ..test_request(root)
    };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .unwrap();

    let error = download_one_with_client(&client, &request, MmdbEdition::Asn)
        .await
        .expect_err("download should fail");

    match error {
        AppError::GeoipDownload {
            edition,
            url,
            reason,
        } => {
            assert_eq!(edition, "GeoLite2-ASN");
            assert!(url.contains("GeoLite2-ASN.mmdb"));
            assert!(reason.contains("404"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn executes_concurrent_downloads() {
    let root = std::env::temp_dir().join("xrat-geoip-download-concurrent-test");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("root should be created");

    let server = spawn_download_server(StatusCode::OK, vec![1_u8, 2, 3]).await;
    let request = DownloadRequest {
        editions: vec![MmdbEdition::Country, MmdbEdition::City, MmdbEdition::Asn],
        url_template: format!("{server}/{{edition}}.mmdb"),
        mmdb_dir: root.clone(),
        ..test_request(root.clone())
    };

    let summary = super::executor::execute_downloads(&request).await;

    assert_eq!(summary.downloaded, 3);
    assert_eq!(summary.skipped, 0);
    assert_eq!(summary.failed.len(), 0);
    assert!(root.join("GeoLite2-Country.mmdb").exists());
    assert!(root.join("GeoLite2-City.mmdb").exists());
    assert!(root.join("GeoLite2-ASN.mmdb").exists());
}

async fn spawn_download_server(status: StatusCode, body: Vec<u8>) -> String {
    use axum::{Router, routing::get};
    use tokio::net::TcpListener;

    let app = Router::new().route(
        "/{file}",
        get(move || {
            let body = body.clone();
            async move { (status, body) }
        }),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://{address}")
}
