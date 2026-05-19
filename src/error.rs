use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("路径信息错误: {0}")]
    Path(String),
    #[error("配置文件解析错误: {0}")]
    Parse(String),
    #[error("IO错误: {0}")]
    FetchIO(#[from] ::std::io::Error),
    #[error("出现未知错误")]
    Unknown(),
}

pub type Result<T> = std::result::Result<T, Error>;
