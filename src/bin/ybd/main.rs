mod app;
mod clap_app;
mod config;
mod directories;
mod cache;

use std::process::ExitCode;

use clap::Parser;
use yumi_bilibili_download::{actuator, error::*};

use crate::{
    app::App,
    clap_app::{Cmd, Commands},
    directories::APP_PATH,
};

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
    let cmd = Cmd::parse();
    let app: App;
    match cmd.subcommand {
        Commands::Login => {
            App::new(false).await?;
        }
        Commands::Download {
            mode,
            batch,
            output,
            url,
        } => {
            let app = App::new(true).await?;
        }
    }
    Ok(true)
}
