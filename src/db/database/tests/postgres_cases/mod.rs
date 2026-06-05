use super::import_cases::test_node;
use super::*;

mod config_cases;
mod connection_test_cases;
mod runtime_session_cases;

pub(super) async fn verify_database_backend(db: &Database) {
    let (first_id, second_id) = config_cases::verify_import_and_config_state(db).await;
    config_cases::verify_reconcile_state(db).await;
    connection_test_cases::verify_connection_test_state(db, second_id).await;
    runtime_session_cases::verify_runtime_session_state(db, second_id).await;

    // Ensure earlier config delete path actually removed the original item.
    assert!(
        db.get_config_by_id(first_id)
            .await
            .expect("deleted query should succeed")
            .is_none()
    );
}
