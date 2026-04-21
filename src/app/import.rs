use crate::db::ImportSource;
use crate::decode::decode_or_raw_text;
use crate::io::read_input;
use crate::model::Node;
use crate::parser::parse_text;

pub fn load_nodes(input: &str) -> Result<(ImportSource, Vec<Node>), Box<dyn std::error::Error>> {
    let (source, input_data) = read_input(input)?;
    let config_text = decode_or_raw_text(&input_data)?;
    reject_raw_json_config(&config_text)?;
    let normalized_text = expand_url_list(&config_text)?;

    Ok((source, parse_text(&normalized_text)))
}

fn reject_raw_json_config(config_text: &str) -> Result<(), Box<dyn std::error::Error>> {
    if serde_json::from_str::<serde_json::Value>(config_text).is_ok() {
        return Err(
            "raw JSON config import is not persisted yet; provide subscription links/text instead"
                .into(),
        );
    }

    Ok(())
}

fn expand_url_list(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut collected = Vec::new();
    let mut saw_url = false;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if looks_like_url(trimmed) {
            saw_url = true;
            let (_, body) = read_input(trimmed)?;
            collected.push(decode_or_raw_text(&body)?);
        } else {
            collected.push(trimmed.to_string());
        }
    }

    if saw_url {
        Ok(collected.join("\n"))
    } else {
        Ok(input.to_string())
    }
}

fn looks_like_url(input: &str) -> bool {
    matches!(
        input.split_once("://").map(|(scheme, _)| scheme),
        Some("http" | "https")
    )
}
