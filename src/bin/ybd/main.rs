mod app;
mod cache;
mod clap_app;
mod config;
mod controller;
mod directories;

use std::process::ExitCode;

use clap::Parser;
use yumi_bilibili_download::{error::*, login};

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
    match cmd.subcommand {
        Commands::Login => {
            let account = login::get_account().await?;
            cache::save_user_info(account, APP_PATH.cache_auth_path())?;
        }
        Commands::Download(args) => {
            let app = App::new().await?;
            controller::start_task(&app, args).await?;
        }
    }
    Ok(true)
}
