use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub fn now_string() -> String {
    now_epoch_seconds().to_string()
}

/// Parse a stored timestamp into epoch seconds. Accepts a bare epoch-seconds
/// string or a civil UTC string like `2026-06-11 12:34:56` / `2026-06-11T12:34:56Z`
/// (any fractional seconds or timezone suffix is ignored). Returns `None` when
/// the value cannot be parsed.
pub fn parse_timestamp_secs(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.bytes().all(|byte| byte.is_ascii_digit()) {
        return trimmed.parse::<i64>().ok();
    }

    let normalized: String = trimmed
        .chars()
        .map(|ch| if ch == 'T' { ' ' } else { ch })
        .collect();
    let (date, time) = normalized.split_once(' ')?;

    let mut date_parts = date.split('-');
    let year: i64 = date_parts.next()?.parse().ok()?;
    let month: i64 = date_parts.next()?.parse().ok()?;
    let day: i64 = date_parts.next()?.parse().ok()?;

    let mut time_parts = time.trim_end_matches('Z').split(':');
    let hour: i64 = time_parts.next()?.parse().ok()?;
    let minute: i64 = time_parts.next()?.parse().ok()?;
    let second: i64 = time_parts
        .next()
        .unwrap_or("0")
        .split(['.', '+'])
        .next()?
        .parse()
        .ok()?;

    let days = days_from_civil(year, month, day);
    Some(days * 86_400 + hour * 3_600 + minute * 60 + second)
}

/// Days since the Unix epoch for a proleptic-Gregorian date (Howard Hinnant's
/// algorithm). Avoids pulling in a date library for one conversion.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::parse_timestamp_secs;

    #[test]
    fn parses_epoch_seconds_string() {
        assert_eq!(parse_timestamp_secs("1000"), Some(1000));
    }

    #[test]
    fn parses_civil_utc_strings() {
        assert_eq!(parse_timestamp_secs("1970-01-01 00:00:00"), Some(0));
        assert_eq!(parse_timestamp_secs("1970-01-02T00:00:00Z"), Some(86_400));
        // fractional seconds and timezone suffixes are ignored.
        assert_eq!(parse_timestamp_secs("1970-01-01 00:00:01.123456"), Some(1));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse_timestamp_secs(""), None);
        assert_eq!(parse_timestamp_secs("not-a-time"), None);
    }
}
