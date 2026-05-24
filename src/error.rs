use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("wrong path info: {0}")]
    Path(String),
    #[error("Configuration file parsing error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    FetchIO(#[from] reqwest::Error),
    #[error("unknown error")]
    Unknown(),
    #[error("norlmal error: {0}")]
    Normal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn default_error_handler(error: Error) {
    eprintln!(
        "\n\tAn error occured in yumi-bilibili-download\n\t{}\n",
        error
    );
}

#[test]
fn error_handle() {
    let pe = Error::Path("nihao".into());
    default_error_handler(pe);
}
