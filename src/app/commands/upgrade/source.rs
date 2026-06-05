use std::path::Path;
use std::process::Command;

use crate::app::AppError;
use crate::app::commands::output;
use crate::cli::UpgradeArgs;

use super::install_binary;

pub(crate) async fn upgrade(args: &UpgradeArgs, target: &Path) -> crate::app::Result<()> {
    let color = output::color_enabled();
    let source_dir = &args.path;
    if !source_dir.join("Cargo.toml").is_file() {
        return Err(AppError::InvalidArgument(format!(
            "no Cargo.toml in {}; pass --path <dir> pointing at the xrat checkout",
            source_dir.display()
        )));
    }

    println!(
        "{}",
        output::notice(
            format!(
                "building from {} (cargo build --release)",
                source_dir.display()
            ),
            color
        )
    );
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(source_dir)
        .status()
        .map_err(|error| AppError::InvalidArgument(format!("cannot run cargo: {error}")))?;
    if !status.success() {
        return Err(AppError::InvalidArgument(format!(
            "cargo build --release failed ({status})"
        )));
    }

    let built = source_dir.join("target").join("release").join("xrat");
    if !built.is_file() {
        return Err(AppError::InvalidArgument(format!(
            "build succeeded but {} is missing",
            built.display()
        )));
    }

    println!(
        "{}",
        output::notice(format!("installing to {}", target.display()), color)
    );
    install_binary(&built, target)?;

    println!("{}", output::success("upgraded xrat from source", color));

    Ok(())
}
