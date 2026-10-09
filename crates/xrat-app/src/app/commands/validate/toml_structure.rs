use super::prelude::*;

const TEST_STAGE_ENUM: &[(&str, &[&str])] = &[
    ("icmp", &["ping"]),
    ("real_delay", &["real-delay"]),
    ("download", &["download-speed"]),
    ("tcp", &[]),
];

const FAILURE_POLICY_ENUM: &[(&str, &[&str])] = &[
    ("continue", &[]),
    ("skip_remaining", &["skip-remaining"]),
    ("mark_failed", &["mark-failed"]),
];

const DATABASE_BACKEND_ENUM: &[(&str, &[&str])] = &[("sqlite", &[]), ("postgres", &[])];

const GEOIP_BACKEND_ENUM: &[(&str, &[&str])] = &[
    ("mmdb", &[]),
    ("ip-whois", &["ipwhois"]),
    ("ip-api", &["ipapi"]),
    ("chain", &[]),
    ("none", &[]),
];

const GEOIP_PROVIDER_ENUM: &[(&str, &[&str])] =
    &[("ip-whois", &["ipwhois"]), ("ip-api", &["ipapi"])];

const TUN_SPLIT_MODE_ENUM: &[(&str, &[&str])] =
    &[("all", &[]), ("blacklist", &[]), ("whitelist", &[])];

/// Field-level checks against the raw TOML value, run before deserializing into
/// `AppConfig`. These catch invalid enum values and wrong types on fields that
/// would otherwise fail deserialization with only a generic parse error.
pub(crate) fn check_known_fields(value: &toml::Value, errors: &mut Vec<Diagnostic>) {
    check_string_array_enum(
        value,
        &["testing", "order"],
        "[testing].order",
        TEST_STAGE_ENUM,
        errors,
    );
    check_scalar_enum(
        value,
        &["testing", "failure_policy"],
        "[testing].failure_policy",
        FAILURE_POLICY_ENUM,
        errors,
    );
    check_duration_field(
        value,
        &["testing", "tcp", "timeout"],
        "[testing.tcp].timeout",
        errors,
    );
    check_status_acceptance_fields(value, errors);
    check_scalar_enum(
        value,
        &["database", "backend"],
        "[database].backend",
        DATABASE_BACKEND_ENUM,
        errors,
    );
    check_scalar_enum(
        value,
        &["testing", "geoip", "backend"],
        "[testing.geoip].backend",
        GEOIP_BACKEND_ENUM,
        errors,
    );
    check_scalar_enum(
        value,
        &["testing", "geoip", "fallback"],
        "[testing.geoip].fallback",
        GEOIP_BACKEND_ENUM,
        errors,
    );
    check_scalar_enum(
        value,
        &["testing", "geoip", "remote", "provider"],
        "[testing.geoip.remote].provider",
        GEOIP_PROVIDER_ENUM,
        errors,
    );
    check_scalar_enum(
        value,
        &["runtime", "tun", "split_mode"],
        "[runtime.tun].split_mode",
        TUN_SPLIT_MODE_ENUM,
        errors,
    );
}

pub(crate) fn check_status_acceptance_fields(value: &toml::Value, errors: &mut Vec<Diagnostic>) {
    let codes = get_path(value, &["testing", "real_delay", "accepted_status_codes"]);
    let ranges = get_path(value, &["testing", "real_delay", "accepted_status_ranges"]);

    if let Some(found) = codes {
        let Some(array) = found.as_array() else {
            errors.push(Diagnostic::new(
                "[testing.real_delay].accepted_status_codes",
                format!("expected an array of integers, got {}", toml_kind(found)),
                "accepted HTTP statuses are configured as numeric status codes.",
                "use an array such as [200, 204].",
            ));
            return;
        };
        for entry in array {
            match entry.as_integer() {
                Some(code) if (100..=599).contains(&code) => {}
                Some(code) => errors.push(Diagnostic::new(
                    "[testing.real_delay].accepted_status_codes",
                    format!("status code {code} is outside 100-599"),
                    "HTTP status acceptance is limited to standard three-digit response classes.",
                    "use status codes from 100 through 599.",
                )),
                None => errors.push(Diagnostic::new(
                    "[testing.real_delay].accepted_status_codes",
                    format!("array entry is not an integer: {entry}"),
                    "accepted HTTP statuses are configured as numeric status codes.",
                    "use an array such as [200, 204].",
                )),
            }
        }
    }

    if let Some(found) = ranges {
        let Some(array) = found.as_array() else {
            errors.push(Diagnostic::new(
                "[testing.real_delay].accepted_status_ranges",
                format!("expected an array of strings, got {}", toml_kind(found)),
                "status ranges use inclusive START-END strings.",
                "use an array such as [\"200-299\", \"300-399\"].",
            ));
            return;
        };
        for entry in array {
            let Some(text) = entry.as_str() else {
                errors.push(Diagnostic::new(
                    "[testing.real_delay].accepted_status_ranges",
                    format!("array entry is not a string: {entry}"),
                    "status ranges use inclusive START-END strings.",
                    "quote each range, for example [\"300-399\"].",
                ));
                continue;
            };
            if let Err(error) = text.parse::<HttpStatusRange>() {
                errors.push(Diagnostic::new(
                    "[testing.real_delay].accepted_status_ranges",
                    error,
                    "status ranges must be ordered and remain within 100-599.",
                    "use inclusive ranges such as \"200-299\" or \"300-399\".",
                ));
            }
        }
    }

    if (codes.is_some() || ranges.is_some())
        && codes
            .and_then(toml::Value::as_array)
            .is_none_or(Vec::is_empty)
        && ranges
            .and_then(toml::Value::as_array)
            .is_none_or(Vec::is_empty)
    {
        errors.push(Diagnostic::new(
            "[testing.real_delay]",
            "configured status acceptance set is empty",
            "a real-delay request could never pass without an accepted status.",
            "add at least one accepted status code or range, or omit both fields to accept 200-299.",
        ));
    }
}

pub(crate) fn get_path<'a>(value: &'a toml::Value, path: &[&str]) -> Option<&'a toml::Value> {
    let mut current = value;
    for segment in path {
        current = current.as_table()?.get(*segment)?;
    }
    Some(current)
}

pub(crate) fn toml_kind(value: &toml::Value) -> &'static str {
    match value {
        toml::Value::String(_) => "a string",
        toml::Value::Integer(_) => "an integer",
        toml::Value::Float(_) => "a float",
        toml::Value::Boolean(_) => "a boolean",
        toml::Value::Datetime(_) => "a datetime",
        toml::Value::Array(_) => "an array",
        toml::Value::Table(_) => "a table",
    }
}

pub(crate) fn enum_value_matches(accepted: &[(&str, &[&str])], text: &str) -> bool {
    accepted
        .iter()
        .any(|(canonical, aliases)| *canonical == text || aliases.contains(&text))
}

pub(crate) fn accepted_values_list(accepted: &[(&str, &[&str])]) -> String {
    accepted
        .iter()
        .map(|(canonical, aliases)| {
            if aliases.is_empty() {
                canonical.to_string()
            } else {
                format!("{canonical} (alias {})", aliases.join(", "))
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn check_scalar_enum(
    value: &toml::Value,
    path: &[&str],
    field: &str,
    accepted: &[(&str, &[&str])],
    errors: &mut Vec<Diagnostic>,
) {
    let Some(found) = get_path(value, path) else {
        return;
    };
    let Some(text) = found.as_str() else {
        errors.push(Diagnostic::new(
            field,
            format!("expected a string, got {}", toml_kind(found)),
            "this field selects one of a fixed set of named options.",
            format!("use one of: {}", accepted_values_list(accepted)),
        ));
        return;
    };
    if !enum_value_matches(accepted, text) {
        errors.push(Diagnostic::new(
            field,
            format!("value \"{text}\" is not accepted here"),
            "this field selects one of a fixed set of named options.",
            format!("accepted values: {}", accepted_values_list(accepted)),
        ));
    }
}

pub(crate) fn check_string_array_enum(
    value: &toml::Value,
    path: &[&str],
    field: &str,
    accepted: &[(&str, &[&str])],
    errors: &mut Vec<Diagnostic>,
) {
    let Some(found) = get_path(value, path) else {
        return;
    };
    let Some(array) = found.as_array() else {
        errors.push(Diagnostic::new(
            field,
            format!("expected an array of strings, got {}", toml_kind(found)),
            "this field lists stages to run.",
            format!("use an array such as [\"{}\"]", accepted[0].0),
        ));
        return;
    };
    for entry in array {
        let Some(text) = entry.as_str() else {
            errors.push(Diagnostic::new(
                field,
                format!("array entry is not a string: {entry}"),
                "each entry names a stage.",
                format!("accepted values: {}", accepted_values_list(accepted)),
            ));
            continue;
        };
        if !enum_value_matches(accepted, text) {
            errors.push(Diagnostic::new(
                field,
                format!("value \"{text}\" is not accepted here"),
                "each entry must name a known stage.",
                format!("accepted values: {}", accepted_values_list(accepted)),
            ));
        }
    }
}

pub(crate) fn check_duration_field(
    value: &toml::Value,
    path: &[&str],
    field: &str,
    errors: &mut Vec<Diagnostic>,
) {
    let Some(found) = get_path(value, path) else {
        return;
    };
    match found {
        toml::Value::Integer(number) if *number < 0 => {
            errors.push(Diagnostic::new(
                field,
                format!("value is {number}"),
                "a duration in milliseconds cannot be negative.",
                "use 0 or a positive number of milliseconds.",
            ));
        }
        toml::Value::Integer(_) => {}
        other => {
            errors.push(Diagnostic::new(
                field,
                format!("expected integer milliseconds, got {}", toml_kind(other)),
                "this field is a duration in milliseconds.",
                "use a plain integer, e.g. 2000.",
            ));
        }
    }
}
