//! TUI 数据模型与状态定义模块

use std::time::SystemTime;

use ybd_core::model::quality::{AudioQuality, VideoEncode, VideoQuality};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
        #[default]
        Account,
        Download,
        Tasks,
        Settings,
}

impl Tab {
        pub fn next(self) -> Self {
                match self {
                        | Tab::Account => Tab::Download,
                        | Tab::Download => Tab::Tasks,
                        | Tab::Tasks => Tab::Settings,
                        | Tab::Settings => Tab::Account,
                }
        }

        pub fn prev(self) -> Self {
                match self {
                        | Tab::Account => Tab::Settings,
                        | Tab::Download => Tab::Account,
                        | Tab::Tasks => Tab::Download,
                        | Tab::Settings => Tab::Tasks,
                }
        }

        #[allow(dead_code)]
        pub fn from_index(index: usize) -> Self {
                match index {
                        | 0 => Tab::Account,
                        | 1 => Tab::Download,
                        | 2 => Tab::Tasks,
                        | 3 => Tab::Settings,
                        | _ => Tab::Account,
                }
        }

        #[allow(dead_code)]
        pub fn to_index(self) -> usize {
                match self {
                        | Tab::Account => 0,
                        | Tab::Download => 1,
                        | Tab::Tasks => 2,
                        | Tab::Settings => 3,
                }
        }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VimMode {
        #[default]
        Normal,
        Insert,
        Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DownloadFocus {
        #[default]
        UrlInput,
        ModeSelect,
        VideoQuality,
        AudioQuality,
        VideoEncode,
        BatchToggle,
        OutputDir,
        FfmpegPath,
}

impl DownloadFocus {
        pub fn next(self) -> Self {
                match self {
                        | DownloadFocus::UrlInput => DownloadFocus::ModeSelect,
                        | DownloadFocus::ModeSelect => DownloadFocus::VideoQuality,
                        | DownloadFocus::VideoQuality => DownloadFocus::AudioQuality,
                        | DownloadFocus::AudioQuality => DownloadFocus::VideoEncode,
                        | DownloadFocus::VideoEncode => DownloadFocus::BatchToggle,
                        | DownloadFocus::BatchToggle => DownloadFocus::OutputDir,
                        | DownloadFocus::OutputDir => DownloadFocus::FfmpegPath,
                        | DownloadFocus::FfmpegPath => DownloadFocus::UrlInput,
                }
        }

        pub fn prev(self) -> Self {
                match self {
                        | DownloadFocus::UrlInput => DownloadFocus::FfmpegPath,
                        | DownloadFocus::ModeSelect => DownloadFocus::UrlInput,
                        | DownloadFocus::VideoQuality => DownloadFocus::ModeSelect,
                        | DownloadFocus::AudioQuality => DownloadFocus::VideoQuality,
                        | DownloadFocus::VideoEncode => DownloadFocus::AudioQuality,
                        | DownloadFocus::BatchToggle => DownloadFocus::VideoEncode,
                        | DownloadFocus::OutputDir => DownloadFocus::BatchToggle,
                        | DownloadFocus::FfmpegPath => DownloadFocus::OutputDir,
                }
        }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsFocus {
        #[default]
        Concurrencies,
        OutputDir,
        FfmpegPath,
}

impl SettingsFocus {
        pub fn next(self) -> Self {
                match self {
                        | SettingsFocus::Concurrencies => SettingsFocus::OutputDir,
                        | SettingsFocus::OutputDir => SettingsFocus::FfmpegPath,
                        | SettingsFocus::FfmpegPath => SettingsFocus::Concurrencies,
                }
        }

        pub fn prev(self) -> Self {
                match self {
                        | SettingsFocus::Concurrencies => SettingsFocus::FfmpegPath,
                        | SettingsFocus::OutputDir => SettingsFocus::Concurrencies,
                        | SettingsFocus::FfmpegPath => SettingsFocus::OutputDir,
                }
        }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DownloadMode {
        #[default]
        Video,
        Audio,
        Cover,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VideoInfoPreview {
        pub bvid: String,
        pub title: String,
        pub owner: String,
        pub duration_secs: u64,
        pub pic: String,
        pub desc: String,
        pub is_collection: bool,
        pub media_count: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
        Downloading,
        Merging,
        Completed,
        Failed(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DownloadTask {
        pub id: usize,
        pub bvid: String,
        pub title: String,
        pub mode: DownloadMode,
        pub status: TaskStatus,
        pub video_downloaded: u64,
        pub video_total: Option<u64>,
        pub audio_downloaded: u64,
        pub audio_total: Option<u64>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UserProfile {
        pub uid: String,
        pub uname: String,
        pub is_login: bool,
        pub vip_type: String,
        pub vip_status_desc: String,
        pub exp_time: Option<SystemTime>,
        pub sessdata_preview: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum QrLoginStatus {
        Idle,
        Generating,
        WaitingScan { qr_text: String, qr_key: String },
        ScannedWaitingConfirm,
        Success,
        Expired,
        Error(String),
}

pub const VIDEO_QUALITIES: [VideoQuality; 7] = [
        VideoQuality::K8,
        VideoQuality::K4,
        VideoQuality::FHD1080P60,
        VideoQuality::FHD1080P,
        VideoQuality::HD720P,
        VideoQuality::SD480P,
        VideoQuality::LD360P,
];

pub const AUDIO_QUALITIES: [AudioQuality; 5] = [
        AudioQuality::HiRes,
        AudioQuality::Dolby,
        AudioQuality::High,
        AudioQuality::Medium,
        AudioQuality::Low,
];

pub const VIDEO_ENCODES: [VideoEncode; 3] = [VideoEncode::AVC, VideoEncode::HEVC, VideoEncode::AV1];
