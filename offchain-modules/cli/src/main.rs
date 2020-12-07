pub mod commands;

use crate::commands::handler;
use crate::commands::types::Opts;
use anyhow::Result;
use clap::Clap;
use tokio::runtime::Builder;

fn main() -> Result<()> {
    env_logger::init();
    let opts: Opts = Opts::parse();
    dbg!(&opts);
    let mut runtime = Builder::new()
        .threaded_scheduler()
        .core_threads(20)
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async { handler(opts).await })
}
