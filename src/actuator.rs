use std::path::{Path, PathBuf};

use clap::ValueEnum;

use crate::error::Error;

#[derive(Debug, Clone, ValueEnum)]
pub enum Mode {
    Cover,
    Audio,
    Vedio,
}

pub fn download_resources(mode: Mode) -> Result<(), Error> {
    match mode {
        Mode::Cover => download_cover(&PathBuf::new(), 1),
        Mode::Audio => download_audio(&PathBuf::new(), 1),
        Mode::Vedio => download_vedio(&PathBuf::new(), 1),
    }
}

fn download_cover(dir: &Path, concurrency: usize) -> Result<(), Error> {
    Ok(())
}

fn download_vedio(dir: &Path, concurrency: usize) -> Result<(), Error> {
    Ok(())
}

fn download_audio(dir: &Path, concurrency: usize) -> Result<(), Error> {
    Ok(())
}
