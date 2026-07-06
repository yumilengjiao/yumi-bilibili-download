use std::io;

use colored::Colorize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
        #[error("wrong path info: {0}")]
        Path(String),
        #[error("Configuration file parsing error: {0}")]
        Parse(#[from] serde_json::Error),
        #[error("IO error: {0}")]
        FetchIO(#[from] reqwest::Error),
        #[error("IO error: {0}")]
        StdFileIO(#[from] io::Error),
        #[error("unknown error")]
        Unknown(),
        #[error("semaphore closed")]
        SemaphoreAcquire(#[from] tokio::sync::AcquireError),
        #[error("error field during build: {0}")]
        Build(String),
        #[error("norlmal error: {0}")]
        Normal(String),
        #[error("mp4meta error: {0}")]
        Mp4metaError(#[from] mp4ameta::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// 统一错误处理
///
/// * `error`: 错误类型
pub fn default_error_handler(error: Error) {
        eprintln!("\n\t{}", "──────────────────────────────────────────".red());
        eprintln!(
                "\t{} {}",
                "✗ Error:".red().bold(),
                error.to_string().yellow()
        );
        eprintln!("\t{}\n", "──────────────────────────────────────────".red());
}

#[test]
fn error_handle() {
        let pe = Error::Path("nihao".into());
        default_error_handler(pe);
}
