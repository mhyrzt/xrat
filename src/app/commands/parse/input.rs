use std::fs;
use std::io::Read;

use crate::app::AppError;
use crate::cli::{ParseArgs, ParseEngine};
use crate::config::{EngineMode, ParsedEntry, parse_links_batch};

use super::json_output::{print_details, print_json};

pub async fn run(args: &ParseArgs) -> crate::app::Result<()> {
    validate_inputs(args)?;
    let inputs = load_inputs(args)?;

    if inputs.is_empty() {
        return Err(AppError::NoSupportedConfig);
    }

    if args.json && inputs.len() != 1 {
        return Err(AppError::InvalidArgument(
            "--json supports exactly one config link".to_string(),
        ));
    }

    let parsed = parse_inputs(&inputs, args.engine)?;

    for (index, entry) in parsed.iter().enumerate() {
        if args.json {
            print_json(&entry.node, entry.engine)?;
        } else {
            if inputs.len() > 1 {
                println!("Config #{}", index + 1);
            }
            print_details(&entry.node, entry.engine);
        }
    }

    Ok(())
}

pub(super) fn parse_inputs(
    inputs: &[ParseInput],
    requested_engine: ParseEngine,
) -> crate::app::Result<Vec<ParsedEntry>> {
    let batch_inputs: Vec<(String, String)> = inputs
        .iter()
        .map(|input| (input.source.clone(), input.link.clone()))
        .collect();
    let parsed = parse_links_batch(&batch_inputs, to_engine_mode(requested_engine))
        .map_err(|error| AppError::InvalidArgument(error.to_string()))?;

    if parsed.is_empty() {
        return Err(AppError::NoSupportedConfig);
    }

    Ok(parsed)
}

pub(super) fn validate_inputs(args: &ParseArgs) -> crate::app::Result<()> {
    let mut modes = 0u8;
    if args.input.is_some() {
        modes += 1;
    }
    if args.file.is_some() {
        modes += 1;
    }
    if args.stdin {
        modes += 1;
    }

    if modes == 0 {
        return Err(AppError::InvalidArgument(
            "provide one input: positional <input>, --file, or --stdin".to_string(),
        ));
    }
    if modes > 1 {
        return Err(AppError::InvalidArgument(
            "use only one input mode: positional <input>, --file, or --stdin".to_string(),
        ));
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub(super) struct ParseInput {
    pub(super) link: String,
    pub(super) source: String,
}

fn load_inputs(args: &ParseArgs) -> crate::app::Result<Vec<ParseInput>> {
    if let Some(input) = &args.input {
        return Ok(vec![ParseInput {
            link: input.trim().to_string(),
            source: "arg".to_string(),
        }]);
    }

    if let Some(path) = &args.file {
        let raw_bytes = fs::read(path)?;
        let decoded = decode_input_text(&raw_bytes, &format!("file {}", path.to_string_lossy()))?;
        return Ok(extract_inputs(
            &decoded,
            &format!("file {}", path.to_string_lossy()),
        ));
    }

    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input)?;
    let decoded = decode_input_text(&input, "stdin")?;
    Ok(extract_inputs(&decoded, "stdin"))
}

pub(super) fn decode_input_text(raw: &[u8], source: &str) -> crate::app::Result<String> {
    crate::support::decode::decode_or_raw_text(raw).map_err(|error| {
        AppError::InvalidArgument(format!("failed to decode {source} input: {error}"))
    })
}

pub(super) fn extract_inputs(input: &str, source_prefix: &str) -> Vec<ParseInput> {
    input
        .lines()
        .enumerate()
        .map(|(index, line)| (index + 1, line.trim()))
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'))
        .map(|(line_no, link)| ParseInput {
            link: link.to_string(),
            source: format!("{source_prefix} line {line_no}"),
        })
        .collect()
}

fn to_engine_mode(engine: ParseEngine) -> EngineMode {
    match engine {
        ParseEngine::Auto => EngineMode::Auto,
        ParseEngine::Xray => EngineMode::Xray,
        ParseEngine::SingBox => EngineMode::SingBox,
    }
}
