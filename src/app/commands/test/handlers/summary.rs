use super::super::*;

pub(super) async fn print_latest_run_summary(
    db: &Database,
    args: &TestArgs,
) -> crate::app::Result<()> {
    let Some(run) = db.get_latest_connection_test_run().await? else {
        println!("No persisted test runs found.");
        return Ok(());
    };
    let tests = db.list_connection_tests_by_run(run.id).await?;
    let tests = filter_latest_run_rows(tests, args.country.as_deref(), args.asn.as_deref());
    let total = tests.len();
    let failed = tests
        .iter()
        .filter(|row| row.failure_kind.is_some())
        .count();
    let ok = total.saturating_sub(failed);
    println!(
        "Latest test run #{} ({}) at {}: total={}, ok={}, failed={}",
        run.id, run.kind, run.created_at, total, ok, failed
    );
    print_geo_distribution(
        "Dial-endpoint country distribution",
        tests
            .iter()
            .filter_map(|row| row.dial_endpoint_country.as_deref()),
    );
    print_geo_distribution(
        "Dial-endpoint ASN distribution",
        tests
            .iter()
            .filter_map(|row| row.dial_endpoint_asn.as_deref()),
    );
    print_geo_distribution(
        "Detected fronting providers (dial endpoint, hint not origin)",
        tests
            .iter()
            .filter_map(|row| row.dial_endpoint_fronting.as_deref()),
    );
    Ok(())
}

pub(super) fn filter_latest_run_rows(
    rows: Vec<crate::db::ConnectionTestRecord>,
    country: Option<&str>,
    asn: Option<&str>,
) -> Vec<crate::db::ConnectionTestRecord> {
    let country = country
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_uppercase());
    let asn = asn
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase());

    rows.into_iter()
        .filter(|row| {
            let country_match = country.as_ref().is_none_or(|filter| {
                row.dial_endpoint_country
                    .as_deref()
                    .map(|value| value.eq_ignore_ascii_case(filter))
                    .unwrap_or(false)
            });
            let asn_match = asn.as_ref().is_none_or(|filter| {
                row.dial_endpoint_asn
                    .as_deref()
                    .map(|value| value.to_ascii_lowercase().contains(filter))
                    .unwrap_or(false)
            });
            country_match && asn_match
        })
        .collect()
}
