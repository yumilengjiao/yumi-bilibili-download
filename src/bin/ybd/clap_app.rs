use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use yumi_bilibili_download::{
    actuator::Mode,
    model::quality::{AudioQuality, VideoEncode, VideoQuality},
};

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
    /// whether to download the collection.The url must include the /ml*** path parameter
    #[arg(short, long)]
    pub batch: bool,
    /// Specify the output directory, which defaults to the current directory.
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// URL of the resources
    pub url: String,
    /// audio quality
    #[arg(long, value_enum)]
    pub quality_audio: Option<AudioQuality>,
    /// video quality
    #[arg(long, value_enum)]
    pub quality_video: Option<VideoQuality>,
    /// Encoding format
    #[arg(long, value_enum)]
    pub encode_video: Option<VideoEncode>,
    /// path of the ffmpeg
    #[arg(long)]
    pub ffmpeg: Option<PathBuf>,
}
