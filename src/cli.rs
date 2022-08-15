use clap::{AppSettings, Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about,
    usage = "esa-cli [FLAGS] <SUBCOMMAND> [<SUB-CMD-OPTIONS>]",
    name = "esa-cli",
    arg_required_else_help = true,
    global_setting(AppSettings::DeriveDisplayOrder)
)]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: SubCommands,
}

#[derive(Subcommand, Debug)]
pub enum SubCommands {
    #[clap(name = "create")]
    Create { post_name: PathBuf },

    #[clap(name = "push")]
    Push { post_name: PathBuf },

    #[clap(
        name = "fetch",
        usage = "esa-cli fetch [FLAGS] [--name <post_name> | --number <post_number>]"
    )]
    Fetch(FetchCommand),

    #[clap(name = "attach")]
    Attach { image_file: PathBuf },
}

#[derive(Args, Debug)]
pub struct FetchCommand {
    #[clap(name = "post_name", long = "name", conflicts_with("post_number"))]
    pub name: Option<PathBuf>,

    #[clap(name = "post_number", long = "number", conflicts_with("post_name"))]
    pub number: Option<u64>,
}
