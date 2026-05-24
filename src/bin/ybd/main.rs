mod app;
mod clap_app;
mod config;
mod directories;

use std::process::ExitCode;

use clap::Parser;
use yumi_bilibili_download::error::*;

use crate::{app::App, clap_app::Cmd};

#[tokio::main]
async fn main() -> ExitCode {
    let result = run().await;
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(err) => {
            default_error_handler(err);
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<bool> {
    // let app = App::new().await?;
    // // println!("{app:#?}");
    let cmd = Cmd::parse();
    println!("{cmd:#?}");
    Ok(true)
}
