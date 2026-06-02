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
    generate_recursive(&cmd, "xrat".to_string(), &args.output)?;
    Ok(())
}

fn generate_recursive(cmd: &clap::Command, name: String, output: &Path) -> crate::app::Result<()> {
    let page = Man::new(cmd.clone()).title(name.clone());
    let path = output.join(format!("{name}.1"));
    let mut file = BufWriter::new(fs::File::create(&path)?);
    page.render(&mut file)?;
    println!("{}", path.display());

    for sub in cmd.get_subcommands() {
        if sub.is_hide_set() {
            continue;
        }
        let sub_name = format!("{}-{}", name, sub.get_name());
        generate_recursive(sub, sub_name, output)?;
    }

    Ok(())
}
