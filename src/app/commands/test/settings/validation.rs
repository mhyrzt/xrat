use super::*;

pub(crate) fn validate_test_stage_order(order: &[ConnectionTestStage]) -> crate::app::Result<()> {
    let mut seen = Vec::with_capacity(order.len());
    for stage in order {
        if seen.contains(stage) {
            return Err(AppError::InvalidArgument(format!(
                "duplicate test stage in [testing].order: {}",
                test_stage_name(*stage)
            )));
        }
        seen.push(*stage);
    }

    Ok(())
}

pub(crate) fn test_stage_name(stage: ConnectionTestStage) -> &'static str {
    stage.config_name()
}

pub(crate) fn resolve_concurrency(value: i32) -> crate::app::Result<usize> {
    if value < 0 {
        return Err(AppError::InvalidArgument(
            "test concurrency must be 0 or greater".to_string(),
        ));
    }

    if value > 0 {
        return Ok(value as usize);
    }

    let parallelism = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(1);
    Ok(parallelism.clamp(1, 8))
}
