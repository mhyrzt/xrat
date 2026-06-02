use crate::app::context::AppContext;
use crate::cli::GeoIpPathArgs;

use super::resolve_mmdb_target_dir;

pub fn run(context: &AppContext, args: &GeoIpPathArgs) -> crate::app::Result<()> {
    println!(
        "{}",
        resolve_mmdb_target_dir(context, args.output.as_ref()).display()
    );
    Ok(())
}
