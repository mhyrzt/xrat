use super::{LocalMmdbLookup, lookup_asn_label, lookup_city_label, lookup_country_iso};
use crate::support::geoip::GeoIpLookup;

#[test]
fn returns_none_for_invalid_inputs() {
    assert!(lookup_country_iso("/tmp/no-such.mmdb".as_ref(), "8.8.8.8").is_none());
    assert!(lookup_city_label("/tmp/no-such.mmdb".as_ref(), "8.8.8.8").is_none());
    assert!(lookup_asn_label("/tmp/no-such.mmdb".as_ref(), "8.8.8.8").is_none());
    assert!(lookup_country_iso("/tmp/no-such.mmdb".as_ref(), "not-ip").is_none());
}

#[test]
fn backend_name_is_mmdb() {
    let lookup = LocalMmdbLookup::new("country.mmdb".into(), "city.mmdb".into(), "asn.mmdb".into());

    assert_eq!(lookup.backend_name(), "mmdb");
}

#[test]
fn looks_up_country_from_real_mmdb_when_provided() {
    let Some(path) = std::env::var_os("XRAT_GEOIP_TEST_MMDB") else {
        return;
    };
    let Some(code) = lookup_country_iso(path.as_ref(), "8.8.8.8") else {
        panic!("expected country code from provided mmdb");
    };
    assert_eq!(code.len(), 2);
    assert!(code.chars().all(|ch| ch.is_ascii_uppercase()));
}

#[test]
fn looks_up_city_from_real_mmdb_when_provided() {
    let Some(path) = std::env::var_os("XRAT_GEOIP_TEST_CITY_MMDB") else {
        return;
    };
    let Some(value) = lookup_city_label(path.as_ref(), "8.8.8.8") else {
        panic!("expected city label from provided mmdb");
    };
    assert!(!value.is_empty());
}

#[test]
fn looks_up_asn_from_real_mmdb_when_provided() {
    let Some(path) = std::env::var_os("XRAT_GEOIP_TEST_ASN_MMDB") else {
        return;
    };
    let Some(value) = lookup_asn_label(path.as_ref(), "8.8.8.8") else {
        panic!("expected asn label from provided mmdb");
    };
    assert!(value.starts_with("AS"));
}
