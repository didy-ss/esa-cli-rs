mod cli;
mod esa_client;
mod post;
mod run;
mod user_config;

use anyhow::{anyhow, Result};
use clap::Parser;
use cli::Cli;
use run::run;
use user_config::UserConfig;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let user_config = UserConfig::from_env().map_err(|e| anyhow!(e))?;

    run(cli.subcommand, user_config)
}
