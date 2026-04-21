use xrat::cli;
use xrat::decode::decode_or_raw_text;
use xrat::io::{read_input, save_json};
use xrat::parser::parse_text;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::parse();
    let input_data = read_input(&args.input)?;
    let config_text = decode_or_raw_text(&input_data)?;

    match serde_json::from_str::<serde_json::Value>(&config_text) {
        Ok(parsed_json) => {
            save_json(&args.output_file, &parsed_json)?;
            println!("Saved raw JSON config directly to: {}", args.output_file.display());
        }
        Err(_) => {
            let normalized_text = expand_url_list(&config_text)?;
            let nodes = parse_text(&normalized_text);
            save_json(&args.output_file, &nodes)?;
            println!(
                "Processed input and saved {} parsed nodes to: {}",
                nodes.len(),
                args.output_file.display()
            );
        }
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
            let body = read_input(trimmed)?;
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
