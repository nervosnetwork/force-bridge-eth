
pub mod commands;

use crate::commands::handler;
use crate::commands::types::Opts;
use anyhow::Result;
use clap::Clap;

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    dbg!(&opts);
    handler(opts)
}
