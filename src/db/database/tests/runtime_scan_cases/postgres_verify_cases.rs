use super::super::postgres_cases::verify_database_backend;
use super::super::*;

#[tokio::test]
async fn verifies_postgres_backend_when_url_is_set() {
    let Ok(url) = std::env::var("XRAT_POSTGRES_TEST_URL") else {
        eprintln!("skipping PostgreSQL verification; set XRAT_POSTGRES_TEST_URL to run it");
        return;
    };
    let db = Database::connect_postgres_url(url)
        .await
        .expect("postgres db should open");

    db.clear_for_test().await.expect("postgres should clear");
    verify_database_backend(&db).await;
    db.clear_for_test().await.expect("postgres should clear");
}
