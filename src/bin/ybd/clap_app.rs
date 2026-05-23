use clap::Parser;
use yumi_bilibili_download::actuator::Mode;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cmd {
    /// the type of resources
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
}
