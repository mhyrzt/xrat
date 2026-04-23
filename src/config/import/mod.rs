mod detect;
mod parsers;
#[allow(dead_code)]
mod subscription;

use crate::model::Node;

#[derive(Debug)]
pub struct ImportResult {
    pub nodes: Vec<Node>,
    pub errors: Vec<(usize, String)>,
    pub metadata: Option<SubscriptionMetadata>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionMetadata {
    pub upload: Option<u64>,
    pub download: Option<u64>,
    pub total: Option<u64>,
    pub expire: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    Auto,
    SingleLink,
    Base64Subscription,
    PlainList,
    Sip008Json,
    XrayJson,
}

pub fn parse_import(
    input: &str,
    mode: ImportMode,
) -> Result<ImportResult, Box<dyn std::error::Error>> {
    let input = input.trim();

    if input.is_empty() {
        return Ok(ImportResult {
            nodes: vec![],
            errors: vec![],
            metadata: None,
        });
    }

    let detected_mode = if mode == ImportMode::Auto {
        detect::detect_format(input)
    } else {
        mode
    };

    match detected_mode {
        ImportMode::SingleLink => parsers::parse_single_link(input),
        ImportMode::Base64Subscription => parsers::parse_base64_subscription(input),
        ImportMode::PlainList => parsers::parse_plain_list(input),
        ImportMode::Sip008Json => parsers::parse_sip008_json(input),
        ImportMode::XrayJson => parsers::parse_xray_json(input),
        ImportMode::Auto => unreachable!("Auto mode should be resolved"),
    }
}
