use xrat::app::path;
use xrat::cli;
use xrat::db::Database;
use xrat::decode::decode_or_raw_text;
use xrat::io::read_input;
use xrat::parser::parse_text;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::parse();
    let app_paths = path::ensure_layout()?;
    let database_path = args
        .database_path
        .unwrap_or_else(|| app_paths.database_path.clone());
    let (source, input_data) = read_input(&args.input)?;
    let config_text = decode_or_raw_text(&input_data)?;
    if serde_json::from_str::<serde_json::Value>(&config_text).is_ok() {
        return Err(
            "raw JSON config import is not persisted yet; provide subscription links/text instead"
                .into(),
        );
    }

    let normalized_text = expand_url_list(&config_text)?;
    let nodes = parse_text(&normalized_text);
    let db = Database::connect(&database_path).await?;
    let summary = db.import_nodes(&source, &nodes).await?;

    println!(
        "Imported {} parsed nodes into {} (subscription #{}, total configs: {})",
        summary.imported_configs,
        database_path.display(),
        summary.subscription_id,
        summary.total_configs
    );

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
