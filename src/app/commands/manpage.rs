use std::fs;
use std::io::BufWriter;
use std::path::Path;

use clap::CommandFactory;
use clap_mangen::Man;

use crate::app::context::AppContext;
use crate::cli::{Cli, ManpageArgs};

pub fn run(_context: &AppContext, args: &ManpageArgs) -> crate::app::Result<()> {
    fs::create_dir_all(&args.output)?;
    let cmd = Cli::command();
    generate_recursive(&cmd, "xrat".to_string(), &args.output, &mut |path| {
        println!("{}", path.display());
    })
}

/// Generate all man pages into `output`, returning the number of pages written.
/// Quiet: used by `setup` so it can report a single summary line.
pub fn generate_into(output: &Path) -> crate::app::Result<usize> {
    fs::create_dir_all(output)?;
    let cmd = Cli::command();
    let mut count = 0usize;
    generate_recursive(&cmd, "xrat".to_string(), output, &mut |_| count += 1)?;
    Ok(count)
}

fn generate_recursive(
    cmd: &clap::Command,
    name: String,
    output: &Path,
    on_written: &mut impl FnMut(&Path),
) -> crate::app::Result<()> {
    let page = Man::new(cmd.clone()).title(name.clone());
    let path = output.join(format!("{name}.1"));
    let mut file = BufWriter::new(fs::File::create(&path)?);
    page.render(&mut file)?;
    on_written(&path);

    for sub in cmd.get_subcommands() {
        if sub.is_hide_set() {
            continue;
        }
        let sub_name = format!("{}-{}", name, sub.get_name());
        generate_recursive(sub, sub_name, output, on_written)?;
    }

    Ok(())
}
