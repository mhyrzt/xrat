use super::*;

pub(super) async fn run_single(
    _args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    config_id: i64,
) -> crate::app::Result<()> {
    let config = context.db.get_config_by_id(config_id).await?;
    let Some(config) = config else {
        tracing::warn!(config_id, "config not found");
        return Ok(());
    };

    let run_id = context
        .db
        .insert_connection_test_run(&ConnectionTestRunInsert {
            kind: "single".to_string(),
        })
        .await?;

    print_single_header(&config);
    let output =
        test_and_record_config(context.db.clone(), config, settings, true, Some(run_id)).await?;
    print_single_summary(&output);
    Ok(())
}
