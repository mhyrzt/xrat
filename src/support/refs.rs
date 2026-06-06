//! Stable, user-facing short references (`ref`) for configs and subscriptions.
//!
//! Refs are random 12-character lowercase hex strings, stable across edits, that
//! give a Docker-like UX (`xrat connect a1b2`) while numeric primary keys stay
//! internal.

/// Number of leading ref characters shown by default in human output.
pub const REF_DISPLAY_LEN: usize = 8;

/// Full ref length stored in the database.
pub const REF_FULL_LEN: usize = 12;

/// Generate a new random 12-character lowercase hex ref.
pub fn generate_ref() -> String {
    // 6 random bytes -> 12 hex chars. Reuse uuid's getrandom-backed v4 source so
    // we don't add a dependency just for randomness.
    let bytes = uuid::Uuid::new_v4().into_bytes();
    let mut out = String::with_capacity(REF_FULL_LEN);
    for byte in &bytes[..REF_FULL_LEN / 2] {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Short display form of a ref (first [`REF_DISPLAY_LEN`] characters).
pub fn short_ref(value: &str) -> &str {
    let end = value
        .char_indices()
        .nth(REF_DISPLAY_LEN)
        .map(|(index, _)| index)
        .unwrap_or(value.len());
    &value[..end]
}

/// Whether `candidate` is a plausible ref prefix: non-empty, all lowercase hex,
/// and no longer than a full ref. Used to disambiguate ref prefixes from numeric
/// ids in CLI arguments.
pub fn is_ref_prefix(candidate: &str) -> bool {
    !candidate.is_empty()
        && candidate.len() <= REF_FULL_LEN
        && candidate
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_twelve_hex_chars() {
        let value = generate_ref();
        assert_eq!(value.len(), REF_FULL_LEN);
        assert!(
            value
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }

    #[test]
    fn generates_distinct_refs() {
        assert_ne!(generate_ref(), generate_ref());
    }

    #[test]
    fn short_ref_takes_first_eight() {
        assert_eq!(short_ref("0123456789ab"), "01234567");
        assert_eq!(short_ref("abc"), "abc");
    }

    #[test]
    fn ref_prefix_detection() {
        assert!(is_ref_prefix("a1b2"));
        assert!(is_ref_prefix("0123456789ab"));
        assert!(!is_ref_prefix(""));
        assert!(!is_ref_prefix("0123456789abc")); // too long
        assert!(!is_ref_prefix("A1B2")); // uppercase
        assert!(!is_ref_prefix("xyz")); // non-hex
    }

    #[test]
    fn numeric_ids_can_look_like_ref_prefixes() {
        // Pure-digit strings are valid hex; callers try numeric id parsing first.
        assert!(is_ref_prefix("42"));
    }
}
