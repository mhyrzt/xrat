use crate::types::{ConfigRow, SubscriptionRow};
use std::cmp::Ordering;

#[derive(Clone, PartialEq)]
pub enum CellValue {
    Text(String),
    Number(f64, &'static str),
    Probe {
        ok: Option<bool>,
        value: Option<f64>,
        detail: Option<String>,
    },
    Outcome {
        state: &'static str,
        detail: Option<String>,
    },
    Empty,
}
impl CellValue {
    fn missing(&self) -> bool {
        matches!(
            self,
            Self::Empty
                | Self::Probe {
                    ok: Some(false),
                    ..
                }
                | Self::Probe {
                    ok: None,
                    value: None,
                    ..
                }
        )
    }
    pub fn compare(&self, other: &Self, descending: bool) -> Ordering {
        match (self.missing(), other.missing()) {
            (true, false) => return Ordering::Greater,
            (false, true) => return Ordering::Less,
            _ => {}
        }
        let order = match (self, other) {
            (Self::Number(a, _), Self::Number(b, _)) => a.total_cmp(b),
            (Self::Probe { value: a, .. }, Self::Probe { value: b, .. }) => {
                a.partial_cmp(b).unwrap_or(Ordering::Equal)
            }
            _ => self.text().to_lowercase().cmp(&other.text().to_lowercase()),
        };
        if descending { order.reverse() } else { order }
    }
    pub fn text(&self) -> String {
        match self {
            Self::Text(value) => value.clone(),
            Self::Number(value, unit) => {
                if *unit == "Mbps" {
                    format!("{value:.2} {unit}")
                } else if unit.is_empty() {
                    format!("{value:.0}")
                } else {
                    format!("{value:.0} {unit}")
                }
            }
            Self::Probe {
                ok: Some(false), ..
            } => "Failed".into(),
            Self::Probe {
                value: Some(value), ..
            } => format!("{value:.0} ms"),
            Self::Probe { ok: Some(true), .. } => "Passed".into(),
            Self::Outcome { state, .. } => match *state {
                "passed" => "Passed",
                "failed" => "Failed",
                _ => "Not tested",
            }
            .into(),
            _ => String::new(),
        }
    }
}
fn text(value: Option<&str>) -> CellValue {
    value
        .filter(|s| !s.is_empty())
        .map(|s| CellValue::Text(s.into()))
        .unwrap_or(CellValue::Empty)
}
fn number(value: Option<f64>, unit: &'static str) -> CellValue {
    value
        .filter(|value| value.is_finite())
        .map(|value| CellValue::Number(value, unit))
        .unwrap_or(CellValue::Empty)
}
pub fn config_name(row: &ConfigRow) -> String {
    if row.name.is_empty() {
        format!("Config #{}", row.id)
    } else {
        row.name.clone()
    }
}
pub fn config_state(row: &ConfigRow) -> &'static str {
    if row.deleted {
        "Deleted"
    } else if row.enabled {
        "Enabled"
    } else {
        "Disabled"
    }
}
pub fn test_state(row: &ConfigRow) -> &'static str {
    let Some(test) = &row.test else {
        return "untested";
    };
    let ok = test.real_delay_ok.or(test.tcp_ok).or(test.icmp_ok);
    if ok == Some(true) {
        "passed"
    } else if ok == Some(false) || test.failure_reason.is_some() {
        "failed"
    } else if test.download_mbps.is_some() || test.upload_mbps.is_some() {
        "passed"
    } else {
        "untested"
    }
}
pub fn config_value(row: &ConfigRow, key: &str, subs: &[SubscriptionRow]) -> CellValue {
    let test = row.test.as_ref();
    match key {
        "name" => CellValue::Text(config_name(row)),
        "state" => CellValue::Text(config_state(row).into()),
        "protocol" => text(Some(&row.protocol)),
        "address" => text(Some(&row.address)),
        "port" => number((row.protocol != "xray-json").then_some(row.port as f64), ""),
        "ref" => text(Some(&row.reference)),
        "subscription" => text(
            row.subscription_id
                .and_then(|id| subs.iter().find(|sub| sub.id == id))
                .map(|sub| sub.name.as_str()),
        ),
        "network" => text(Some(&row.network)),
        "tls" => text(row.tls.as_deref()),
        "sni" => text(row.sni.as_deref()),
        "country" => text(row.country.as_deref()),
        "asn" => text(row.asn.as_deref()),
        "updated" => text(Some(&row.updated_at)),
        "tested" => text(test.and_then(|test| test.tested_at.as_deref())),
        "test" => CellValue::Outcome {
            state: test_state(row),
            detail: test.and_then(|test| test.failure_reason.clone()),
        },
        "delay" | "tcp" | "icmp" => {
            let (ok, value) = test
                .map(|test| match key {
                    "tcp" => (test.tcp_ok, test.tcp_ms),
                    "icmp" => (test.icmp_ok, test.icmp_ms),
                    _ => (test.real_delay_ok, test.real_delay_ms),
                })
                .unwrap_or((None, None));
            CellValue::Probe {
                ok,
                value: value.map(|value| value as f64),
                detail: test.and_then(|test| test.failure_reason.clone()),
            }
        }
        "download" => number(test.and_then(|test| test.download_mbps), "Mbps"),
        "upload" => number(test.and_then(|test| test.upload_mbps), "Mbps"),
        "connect" => number(
            test.and_then(|test| test.connect_ms)
                .map(|value| value as f64),
            "ms",
        ),
        "ttfb" => number(
            test.and_then(|test| test.ttfb_ms).map(|value| value as f64),
            "ms",
        ),
        "http" => number(
            test.and_then(|test| test.http_status)
                .map(|value| value as f64),
            "",
        ),
        _ => CellValue::Empty,
    }
}
pub fn source_label(source: &str) -> String {
    match source {
        "url" => "URL".into(),
        "file" => "File".into(),
        "raw_text" => "Imported text".into(),
        other => other.replace('_', " "),
    }
}
pub fn subscription_value(row: &SubscriptionRow, key: &str) -> CellValue {
    match key {
        "name" => text(Some(&row.name)),
        "source" => CellValue::Text(source_label(&row.source)),
        "count" => number(Some(row.config_count as f64), ""),
        "created" => text(Some(&row.created_at)),
        "updated" => text(Some(&row.updated_at)),
        "ref" => text(Some(&row.reference)),
        _ => CellValue::Empty,
    }
}
