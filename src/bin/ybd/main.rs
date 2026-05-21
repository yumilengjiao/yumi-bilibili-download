mod app;
mod clap_app;
mod config;
mod directories;

use std::process::ExitCode;

use clap::Parser;
use yumi_bilibili_download::error::*;

use crate::clap_app::Cmd;

fn main() -> ExitCode {
    let result = run();
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(_) => ExitCode::FAILURE,
    }
}

fn run() -> Result<bool> {
    let cmd = Cmd::parse();
    println!("{cmd:#?}");
    Ok(true)
}
