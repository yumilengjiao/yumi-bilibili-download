use std::{fs::File, num::NonZeroUsize, path::Path, thread};

use serde::{Deserialize, Serialize};
use yumi_bilibili_download::error::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
        #[serde(default = "default_concurrencies")]
        pub concurrencies: usize,
}

impl Config {
        /// Err: 读取配置文件时出现解析错误
        ///
        /// * `config_file_path`: [配置文件路径]
        pub fn new(config_file_path: Option<&Path>) -> Result<Self> {
                match config_file_path {
                        | Some(file_path) => read_from_local(file_path),
                        | None => Ok(Config {
                                concurrencies: default_concurrencies(),
                        }),
                }
        }
}

fn read_from_local(config_path: &Path) -> Result<Config> {
        match File::open(config_path) {
                | Ok(config_file) => {
                        let config: Config = serde_json::from_reader(config_file)?;
                        Ok(config)
                },
                | Err(_) => Ok(Config {
                        concurrencies: default_concurrencies(),
                }),
        }
}

fn default_concurrencies() -> usize {
        thread::available_parallelism()
                .unwrap_or(NonZeroUsize::new(1).unwrap())
                .get()
}
