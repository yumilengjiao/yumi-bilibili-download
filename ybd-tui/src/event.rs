//! 异步事件定义与轮询通道

use crossterm::event::KeyEvent;
use ybd_core::model::account::Account;

use crate::model::{DownloadMode, QrLoginStatus, UserProfile, VideoInfoPreview};

pub enum AppEvent {
        #[allow(dead_code)]
        Key(KeyEvent),
        Tick,
        #[allow(dead_code)]
        Resize(u16, u16),
        // 扫码登录事件
        QrStatusUpdate(QrLoginStatus),
        LoginSuccess(Account),
        UserProfileLoaded(UserProfile),
        // 解析视频事件
        VideoInfoParsed(Result<VideoInfoPreview, String>),
        // 下载任务事件
        TaskCreated {
                task_id: usize,
                bvid: String,
                title: String,
                mode: DownloadMode,
        },
        TaskVideoProgress {
                task_id: usize,
                downloaded: u64,
                total: Option<u64>,
        },
        TaskAudioProgress {
                task_id: usize,
                downloaded: u64,
                total: Option<u64>,
        },
        TaskMerging {
                task_id: usize,
        },
        TaskCompleted {
                task_id: usize,
        },
        TaskFailed {
                task_id: usize,
                error: String,
        },
        // 全局通知提示
        #[allow(dead_code)]
        Notify(String),
}
