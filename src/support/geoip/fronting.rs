//! Conservative CDN/relay/fronting provider detection.
//!
//! Matching a dialed IP to a known CDN or relay network is a hint, not proof:
//! it means the address xrat connected to belongs to a fronting provider, so
//! the real proxy origin may be in a different country/ASN or hidden entirely.

/// Known fronting/CDN/relay networks, matched as case-insensitive substrings of
/// the ASN organization label (e.g. `AS13335 CLOUDFLARENET`). The list is kept
/// conservative and provider-named so callers can show a precise hint.
const PROVIDERS: &[(&str, &str)] = &[
    ("CLOUDFLARE", "cloudflare"),
    ("FASTLY", "fastly"),
    ("AKAMAI", "akamai"),
    ("CLOUDFRONT", "cloudfront"),
    ("AMAZON", "amazon"),
    ("GOOGLE", "google"),
    ("MICROSOFT", "microsoft"),
    ("AZURE", "azure"),
    ("GCORE", "gcore"),
    ("G-CORE", "gcore"),
    ("BUNNY", "bunnycdn"),
    ("STACKPATH", "stackpath"),
    ("INCAPSULA", "imperva"),
    ("IMPERVA", "imperva"),
    ("SUCURI", "sucuri"),
    ("DDOS-GUARD", "ddos-guard"),
    ("DDOSGUARD", "ddos-guard"),
    ("CDN77", "cdn77"),
    ("LIMELIGHT", "edgio"),
    ("EDGIO", "edgio"),
];

/// Return a provider label when the ASN organization names a known CDN/relay
/// network, or `None` otherwise.
pub fn detect_fronting(asn: Option<&str>) -> Option<String> {
    let asn = asn?.to_ascii_uppercase();
    PROVIDERS
        .iter()
        .find(|(needle, _)| asn.contains(needle))
        .map(|(_, label)| (*label).to_string())
}

#[cfg(test)]
mod tests {
    use super::detect_fronting;

    #[test]
    fn detects_cloudflare_from_asn_label() {
        assert_eq!(
            detect_fronting(Some("AS13335 CLOUDFLARENET")).as_deref(),
            Some("cloudflare")
        );
    }

    #[test]
    fn is_case_insensitive() {
        assert_eq!(
            detect_fronting(Some("as54113 fastly inc")).as_deref(),
            Some("fastly")
        );
    }

    #[test]
    fn returns_none_for_unknown_or_missing_asn() {
        assert_eq!(detect_fronting(Some("AS9009 M247 LTD")), None);
        assert_eq!(detect_fronting(None), None);
    }
}
