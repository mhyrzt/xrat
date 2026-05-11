use super::super::{ImportParseError, ImportResult, SubscriptionMetadata};

pub fn parse_plain_list(input: &str) -> Result<ImportResult, ImportParseError> {
    let mut nodes = Vec::new();
    let mut errors = Vec::new();
    let mut metadata = SubscriptionMetadata {
        upload: None,
        download: None,
        total: None,
        expire: None,
        status: None,
    };

    for (line_num, line) in input.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with("STATUS=") {
            metadata.status = Some(line.trim_start_matches("STATUS=").to_string());
            continue;
        }

        match crate::config::line::parse_line(line) {
            Some(mut node) => {
                crate::config::normalize::normalize(&mut node);
                nodes.push(node);
            }
            None => errors.push((
                line_num + 1,
                format!(
                    "Failed to parse line: {}",
                    line.chars().take(50).collect::<String>()
                ),
            )),
        }
    }

    let metadata = if metadata.status.is_some() {
        Some(metadata)
    } else {
        None
    };

    Ok(ImportResult {
        nodes,
        errors,
        metadata,
    })
}
