//! `zeron-remote` — a thin CLI over the local Zeron MCP tools.

use clap::Parser;
use zeron_remote::cli::Cli;
use zeron_remote::run::run;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(run(cli))
}
