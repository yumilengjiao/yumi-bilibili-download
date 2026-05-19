mod app;

use std::process::ExitCode;

use yumi_bilibili_download::error::*;

fn main() -> ExitCode {
    let result = run();
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(_) => ExitCode::FAILURE,
    }
}

fn run() -> Result<bool> {
    Ok(true)
}
