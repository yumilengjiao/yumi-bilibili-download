use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use yumi_bilibili_download::actuator::Mode;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cmd {
    #[command(subcommand)]
    pub subcommand: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// download subcommand
    Download(DownloadArgs),
    /// sign in your account of bilibili
    Login,
}

#[derive(Args, Debug)]
pub struct DownloadArgs {
    /// the type of resource
    #[arg(value_enum)]
    pub mode: Mode,
    /// whether to download the collection
    #[arg(short, long)]
    pub batch: bool,
    /// Specify the output directory, which defaults to the current directory.
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// URL of the resources
    pub url: String,
}
