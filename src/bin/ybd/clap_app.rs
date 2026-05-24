use clap::{Parser, Subcommand};
use yumi_bilibili_download::actuator::Mode;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cmd {
    #[command(subcommand)]
    subcommand: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// download subcommand
    Download {
        /// the type of resource
        #[arg(value_enum)]
        mode: Mode,
        /// whether to download the collection
        #[arg(short, long)]
        batch: bool,
        /// Specify the output directory, which defaults to the current directory.
        #[arg(short, long)]
        output: Option<String>,
        /// URL of the resources
        url: String,
    },
    /// sign in your account of bilibili
    Login,
}
