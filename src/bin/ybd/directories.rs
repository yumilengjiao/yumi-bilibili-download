use std::{
    env,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use etcetera::BaseStrategy;

pub static APP_PATH: LazyLock<AppPath> =
    LazyLock::new(|| AppPath::new().expect("Unable to get home directory"));

pub struct AppPath {
    vedio_dir: PathBuf,
    audio_dir: PathBuf,
    cover_dir: PathBuf,
    config_path: PathBuf,
}

impl AppPath {
    pub fn new() -> Option<Self> {
        let base_dir = etcetera::choose_base_strategy().ok()?;

        // 下载视频文件的默认下载目录
        let vedio_dir = if let Some(os_vedio_path) = env::var_os("VEDIO_DIR") {
            PathBuf::from(os_vedio_path)
        } else {
            env::current_dir().ok()?
        };

        // 下载音频文件的默认下载目录
        let audio_dir = if let Some(os_audio_path) = env::var_os("AUDIO_DIR") {
            PathBuf::from(os_audio_path)
        } else {
            env::current_dir().ok()?
        };

        // 下载封面文件的默认下载目录
        let cover_dir = if let Some(os_cover_path) = env::var_os("VEDIO_DIR") {
            PathBuf::from(os_cover_path)
        } else {
            env::current_dir().ok()?
        };

        // 配置文件目录
        let config_path = if let Some(os_config_path) = env::var_os("CONFIG_PATH") {
            PathBuf::from(os_config_path)
        } else {
            base_dir.config_dir().join("ybd")
        };
        Some(Self {
            vedio_dir,
            audio_dir,
            cover_dir,
            config_path,
        })
    }
    pub fn vedio_dir(&self) -> &Path {
        &self.vedio_dir
    }

    pub fn audio_dir(&self) -> &Path {
        &self.audio_dir
    }

    pub fn cover_dir(&self) -> &Path {
        &self.cover_dir
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }
}
