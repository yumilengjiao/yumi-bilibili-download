//! 跨平台路径解析模块
//!
//! 管理 ybd-tui 的配置路径、缓存凭证路径及默认下载输出目录。

use std::{
        env,
        path::{Path, PathBuf},
        sync::LazyLock,
};

use etcetera::BaseStrategy;

pub static APP_PATH: LazyLock<AppPath> =
        LazyLock::new(|| AppPath::new().expect("无法获取系统标准主目录"));

pub struct AppPath {
        video_dir: PathBuf,
        audio_dir: PathBuf,
        cover_dir: PathBuf,
        config_path: PathBuf,
        cache_auth_path: PathBuf,
}

impl AppPath {
        pub fn new() -> Option<Self> {
                let base_dir = etcetera::choose_base_strategy().ok()?;

                let video_dir = if let Some(os_video_path) = env::var_os("VIDEO_DIR") {
                        PathBuf::from(os_video_path)
                } else {
                        env::current_dir().ok()?
                };

                let audio_dir = if let Some(os_audio_path) = env::var_os("AUDIO_DIR") {
                        PathBuf::from(os_audio_path)
                } else {
                        env::current_dir().ok()?
                };

                let cover_dir = if let Some(os_cover_path) = env::var_os("COVER_DIR") {
                        PathBuf::from(os_cover_path)
                } else {
                        env::current_dir().ok()?
                };

                let config_path = if let Some(os_config_path) = env::var_os("CONFIG_PATH") {
                        PathBuf::from(os_config_path)
                } else {
                        base_dir.config_dir().join("ybd").join("config.json")
                };

                let cache_auth_path = if let Some(os_auth_path) = env::var_os("AUTH_PATH") {
                        PathBuf::from(os_auth_path)
                } else {
                        base_dir.cache_dir().join("ybd").join("auth.json")
                };

                Some(Self {
                        video_dir,
                        audio_dir,
                        cover_dir,
                        config_path,
                        cache_auth_path,
                })
        }

        pub fn video_dir(&self) -> &Path {
                &self.video_dir
        }

        #[allow(dead_code)]
        pub fn audio_dir(&self) -> &Path {
                &self.audio_dir
        }

        #[allow(dead_code)]
        pub fn cover_dir(&self) -> &Path {
                &self.cover_dir
        }

        pub fn config_path(&self) -> &Path {
                &self.config_path
        }

        pub fn cache_auth_path(&self) -> &Path {
                &self.cache_auth_path
        }
}
