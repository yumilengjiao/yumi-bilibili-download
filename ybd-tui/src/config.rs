//! 本地配置解析模块
//!
//! 负责从配置文件加载并发数、默认存储路径、画质/音质/编码偏好等设置。

use std::{fs::File, num::NonZeroUsize, path::Path, thread};

use serde::{Deserialize, Serialize};
use ybd_core::error::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
        #[serde(default = "default_concurrencies")]
        pub concurrencies: usize,
        #[serde(default)]
        pub output_dir: Option<String>,
        #[serde(default)]
        pub ffmpeg_path: Option<String>,
        #[serde(default)]
        pub download_mode: Option<usize>,
        #[serde(default)]
        pub video_quality_idx: Option<usize>,
        #[serde(default)]
        pub audio_quality_idx: Option<usize>,
        #[serde(default)]
        pub video_encode_idx: Option<usize>,
}

impl Config {
        pub fn new(config_file_path: Option<&Path>) -> Result<Self> {
                match config_file_path {
                        Some(file_path) => read_from_local(file_path),
                        None => Ok(Config {
                                concurrencies: default_concurrencies(),
                                output_dir: None,
                                ffmpeg_path: None,
                                download_mode: None,
                                video_quality_idx: None,
                                audio_quality_idx: None,
                                video_encode_idx: None,
                        }),
                }
        }

        pub fn save(
                &self,
                config_path: &Path,
        ) -> Result<()> {
                if let Some(parent) = config_path.parent() {
                        std::fs::create_dir_all(parent)?;
                }
                let file = File::create(config_path)?;
                serde_json::to_writer_pretty(file, self)?;
                Ok(())
        }
}

fn read_from_local(config_path: &Path) -> Result<Config> {
        match File::open(config_path) {
                Ok(config_file) => {
                        let config: Config = serde_json::from_reader(config_file)?;
                        Ok(config)
                },
                Err(_) => Ok(Config {
                        concurrencies: default_concurrencies(),
                        output_dir: None,
                        ffmpeg_path: None,
                        download_mode: None,
                        video_quality_idx: None,
                        audio_quality_idx: None,
                        video_encode_idx: None,
                }),
        }
}

fn default_concurrencies() -> usize {
        thread::available_parallelism()
                .unwrap_or(NonZeroUsize::new(1).unwrap())
                .get()
}
