use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::cli::InitArgs;

const DEFAULT_CONFIG_CONTENTS: &str = "# XRAT configuration\n\n";

const DEFAULT_CONFIG_TEMPLATE: &str = include_str!("init_default_config.toml");

pub fn run(context: &AppContext, args: &InitArgs) -> crate::app::Result<()> {
    let root = &context.runtime_paths.root_dir;
    let config_path = &context.runtime_paths.config_path;
    let db_path = &context.runtime_paths.database_path;
    let runtime_dir = root.join("runtime");
    let logs_dir = root.join("logs");
    let mmdb_dir = root.join("mmdb");

    let mut created = Vec::new();
    let mut present = Vec::new();

    if args.dry_run {
        println!(
            "{}",
            output::notice("Dry run: no files written.", output::color_enabled())
        );
        println!(
            "{}",
            output::format_kv(
                Some("Would create if absent"),
                &[
                    ("root", format!("{}/", root.display())),
                    ("config", config_path.display().to_string()),
                    ("database", db_path.display().to_string()),
                    ("runtime", format!("{}/", runtime_dir.display())),
                    ("logs", format!("{}/", logs_dir.display())),
                    ("mmdb", format!("{}/", mmdb_dir.display())),
                ],
                output::color_enabled(),
            )
        );
        return Ok(());
    }

    if root.exists() {
        present.push(format!("{}/", root.display()));
    } else {
        std::fs::create_dir_all(root)?;
        created.push(format!("{}/", root.display()));
    }

    if config_path.exists() {
        let current = std::fs::read_to_string(config_path).unwrap_or_default();
        if current == DEFAULT_CONFIG_CONTENTS {
            std::fs::write(config_path, DEFAULT_CONFIG_TEMPLATE)?;
            created.push(format!(
                "{} (written default template)",
                config_path.display()
            ));
        } else {
            present.push(format!("{} (already customized)", config_path.display()));
        }
    } else {
        std::fs::write(config_path, DEFAULT_CONFIG_TEMPLATE)?;
        created.push(config_path.display().to_string());
    }

    present.push(format!("{} (database ready)", db_path.display()));

    for (dir, _label) in [
        (&runtime_dir, "runtime/"),
        (&logs_dir, "logs/"),
        (&mmdb_dir, "mmdb/"),
    ] {
        if dir.exists() {
            present.push(format!("{}/", dir.display()));
        } else {
            std::fs::create_dir_all(dir)?;
            created.push(format!("{}/", dir.display()));
        }
    }

    println!(
        "{}",
        output::success("xrat initialized successfully.", output::color_enabled())
    );

    if !created.is_empty() {
        println!();
        println!(
            "{}",
            output::format_list("Created", &created, output::color_enabled())
        );
    }

    if !present.is_empty() {
        println!();
        println!(
            "{}",
            output::format_list("Already present", &present, output::color_enabled())
        );
    }

    println!();
    println!(
        "{}",
        output::format_list(
            "Next steps",
            &[
                "xrat import <subscription-url>".to_string(),
                "xrat list configs".to_string(),
            ],
            output::color_enabled(),
        )
    );

    Ok(())
}
