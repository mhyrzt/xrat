use super::super::super::*;

#[test]
fn resolves_endpoint_meta_priority_with_real_mmdb_when_provided() {
    let Some(city_path) = std::env::var_os("XRAT_GEOIP_TEST_CITY_MMDB") else {
        return;
    };
    let Some(country_path) = std::env::var_os("XRAT_GEOIP_TEST_MMDB") else {
        return;
    };
    let Some(asn_path) = std::env::var_os("XRAT_GEOIP_TEST_ASN_MMDB") else {
        return;
    };

    let ip = "8.8.8.8";
    let meta = resolve_endpoint_meta(
        Some(ip),
        true,
        city_path.as_ref(),
        country_path.as_ref(),
        asn_path.as_ref(),
    );

    if let Some(city) = geoip::lookup_city_label(city_path.as_ref(), ip) {
        assert_eq!(meta.location.as_deref(), Some(city.as_str()));
        assert_eq!(
            meta.country.as_deref(),
            city.split('/').next().map(str::trim)
        );
        return;
    }

    if let Some(country) = geoip::lookup_country_iso(country_path.as_ref(), ip) {
        assert_eq!(meta.location.as_deref(), Some(country.as_str()));
        assert_eq!(meta.country.as_deref(), Some(country.as_str()));
        return;
    }

    if let Some(asn) = geoip::lookup_asn_label(asn_path.as_ref(), ip) {
        assert_eq!(meta.location.as_deref(), Some(asn.as_str()));
        assert!(meta.country.is_none());
        return;
    }

    panic!("expected at least one mmdb lookup to resolve for provided test assets");
}
