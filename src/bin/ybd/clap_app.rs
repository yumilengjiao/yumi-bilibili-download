use clap::{Parser, ValueEnum};

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

#[derive(Debug, Clone, ValueEnum)]
enum Mode {
    Cover,
    Audio,
    Vedio,
}
