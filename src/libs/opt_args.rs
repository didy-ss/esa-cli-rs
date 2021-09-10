use std::path::PathBuf;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    author,
    about,
    global_settings = &[AppSettings::ColoredHelp, AppSettings::DeriveDisplayOrder],
    usage = "esa-cli [FLAGS] <SUBCOMMAND> [<SUB-CMD-OPTIONS>]",
    name = "esa-cli"
)]
pub struct Opts {
    #[structopt(subcommand)]
    pub sub: SubOpts,
}

#[derive(StructOpt, Debug)]
pub enum SubOpts {
    #[structopt(
        name = "new",
        settings(&[AppSettings::ColoredHelp, AppSettings::DeriveDisplayOrder])
    )]
    New { post_name: PathBuf },

    #[structopt(
        name = "post",
        settings(&[AppSettings::ColoredHelp, AppSettings::DeriveDisplayOrder])
    )]
    Post { post_name: PathBuf },

    #[structopt(
        name = "get",
        usage = "esa-cli-get [FLAGS] [--name <post_name> | --number <post_number>]",
        settings(&[AppSettings::ColoredHelp, AppSettings::DeriveDisplayOrder])
    )]
    Get(GetOpt),

    #[structopt(
        name = "attach",
        settings(&[AppSettings::ColoredHelp, AppSettings::DeriveDisplayOrder])
    )]
    Attach { file_path: PathBuf },
}

#[derive(StructOpt, Debug)]
pub struct GetOpt {
    #[structopt(name = "post_name", long = "name", conflicts_with_all(&["number", "all"]))]
    pub name: Option<PathBuf>,

    #[structopt(name = "post_number", long = "number", conflicts_with_all(&["name", "all"]))]
    pub number: Option<u64>,
}
