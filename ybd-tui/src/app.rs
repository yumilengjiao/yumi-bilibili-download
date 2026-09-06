//! 应用程序核心状态模块
//!
//! 管理全量 TUI 交互状态、Vim 模式、路由表单、后台任务列表与会话凭据。

use std::path::PathBuf;

use tokio::sync::mpsc::UnboundedSender;
use ybd_core::model::account::Account;

use crate::{
        cache::{load_user_from_file, remove_user_info, save_user_info},
        config::Config,
        directories::APP_PATH,
        event::AppEvent,
        model::{
                AUDIO_QUALITIES, DownloadFocus, DownloadMode, DownloadTask, QrLoginStatus,
                SettingsFocus, Tab, TaskStatus, UserProfile, VIDEO_ENCODES, VIDEO_QUALITIES,
                VideoInfoPreview, VimMode,
        },
        service,
};

pub struct App {
        pub tab: Tab,
        pub mode: VimMode,
        pub show_help: bool,
        pub should_quit: bool,
        pub config: Config,
        pub account: Option<Account>,
        pub user_profile: Option<UserProfile>,
        pub qr_status: QrLoginStatus,
        // 下载表单状态
        pub url_input: String,
        pub cursor_pos: usize,
        pub download_focus: DownloadFocus,
        pub download_mode: DownloadMode,
        pub video_quality_idx: usize,
        pub audio_quality_idx: usize,
        pub video_encode_idx: usize,
        pub batch: bool,
        pub output_dir_input: String,
        pub ffmpeg_path_input: String,
        pub preview_info: Option<VideoInfoPreview>,
        pub is_parsing: bool,
        pub parse_error: Option<String>,
        // 任务列表
        pub tasks: Vec<DownloadTask>,
        pub task_counter: usize,
        pub selected_task_idx: usize,
        // 设置表单状态
        pub settings_focus: SettingsFocus,
        pub concurrencies_input: String,
        pub settings_output_input: String,
        pub settings_ffmpeg_input: String,
        // 命令与通知
        pub command_input: String,
        pub status_msg: Option<String>,
        pub tx: UnboundedSender<AppEvent>,
}

impl App {
        pub fn new(tx: UnboundedSender<AppEvent>) -> Self {
                let config = Config::new(Some(APP_PATH.config_path())).unwrap_or_else(|_| Config {
                        concurrencies: 4,
                        output_dir: None,
                        ffmpeg_path: None,
                });

                let account = load_user_from_file(APP_PATH.cache_auth_path()).ok();

                let output_dir_input = config
                        .output_dir
                        .clone()
                        .unwrap_or_else(|| APP_PATH.video_dir().to_string_lossy().to_string());
                let ffmpeg_path_input = config.ffmpeg_path.clone().unwrap_or_default();

                let concurrencies_input = config.concurrencies.to_string();
                let settings_output_input = output_dir_input.clone();
                let settings_ffmpeg_input = ffmpeg_path_input.clone();

                let app = Self {
                        tab: Tab::Account,
                        mode: VimMode::Normal,
                        show_help: false,
                        should_quit: false,
                        config,
                        account,
                        user_profile: None,
                        qr_status: QrLoginStatus::Idle,
                        url_input: String::new(),
                        cursor_pos: 0,
                        download_focus: DownloadFocus::UrlInput,
                        download_mode: DownloadMode::Video,
                        video_quality_idx: 2, // 默认 1080P60
                        audio_quality_idx: 0, // 默认 Hi-Res
                        video_encode_idx: 0,  // 默认 AVC
                        batch: false,
                        output_dir_input,
                        ffmpeg_path_input,
                        preview_info: None,
                        is_parsing: false,
                        parse_error: None,
                        tasks: Vec::new(),
                        task_counter: 0,
                        selected_task_idx: 0,
                        settings_focus: SettingsFocus::Concurrencies,
                        concurrencies_input,
                        settings_output_input,
                        settings_ffmpeg_input,
                        command_input: String::new(),
                        status_msg: None,
                        tx: tx.clone(),
                };

                if let Some(ref acc) = app.account {
                        let acc_clone = acc.clone();
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                                service::fetch_user_profile(&acc_clone, tx_clone).await;
                        });
                }

                app
        }

        pub fn handle_event(
                &mut self,
                event: AppEvent,
        ) {
                match event {
                        | AppEvent::UserProfileLoaded(profile) => {
                                self.user_profile = Some(profile);
                        },
                        | AppEvent::QrStatusUpdate(status) => {
                                self.qr_status = status;
                        },
                        | AppEvent::LoginSuccess(acc) => {
                                let _ = save_user_info(&acc, APP_PATH.cache_auth_path());
                                self.account = Some(acc);
                                self.qr_status = QrLoginStatus::Success;
                                self.set_status("扫码登录成功！");
                                if let Some(ref a) = self.account {
                                        let acc_clone = a.clone();
                                        let tx_clone = self.tx.clone();
                                        tokio::spawn(async move {
                                                service::fetch_user_profile(&acc_clone, tx_clone)
                                                        .await;
                                        });
                                }
                        },
                        | AppEvent::VideoInfoParsed(res) => {
                                self.is_parsing = false;
                                match res {
                                        | Ok(info) => {
                                                if info.is_collection {
                                                        self.batch = true;
                                                }
                                                self.set_status(format!(
                                                        "成功解析: {}",
                                                        info.title
                                                ));
                                                self.preview_info = Some(info);
                                                self.parse_error = None;
                                        },
                                        | Err(err) => {
                                                self.parse_error = Some(err.clone());
                                                self.set_status(format!("解析失败: {}", err));
                                        },
                                }
                        },
                        | AppEvent::TaskCreated {
                                task_id,
                                bvid,
                                title,
                                mode,
                        } => {
                                self.tasks.push(DownloadTask {
                                        id: task_id,
                                        bvid,
                                        title,
                                        mode,
                                        status: TaskStatus::Downloading,
                                        video_downloaded: 0,
                                        video_total: None,
                                        audio_downloaded: 0,
                                        audio_total: None,
                                });
                        },
                        | AppEvent::TaskVideoProgress {
                                task_id,
                                downloaded,
                                total,
                        } => {
                                if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                                        t.video_downloaded = downloaded;
                                        t.video_total = total;
                                }
                        },
                        | AppEvent::TaskAudioProgress {
                                task_id,
                                downloaded,
                                total,
                        } => {
                                if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                                        t.audio_downloaded = downloaded;
                                        t.audio_total = total;
                                }
                        },
                        | AppEvent::TaskMerging { task_id } => {
                                if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                                        t.status = TaskStatus::Merging;
                                }
                        },
                        | AppEvent::TaskCompleted { task_id } => {
                                let mut title = String::new();
                                if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                                        t.status = TaskStatus::Completed;
                                        title = t.title.clone();
                                }
                                if !title.is_empty() {
                                        self.set_status(format!("任务 [{}] 下载完成", title));
                                }
                        },
                        | AppEvent::TaskFailed { task_id, error } => {
                                let mut title = String::new();
                                if let Some(t) = self.tasks.iter_mut().find(|t| t.id == task_id) {
                                        t.status = TaskStatus::Failed(error.clone());
                                        title = t.title.clone();
                                }
                                if !title.is_empty() {
                                        self.set_status(format!(
                                                "任务 [{}] 失败: {}",
                                                title, error
                                        ));
                                }
                        },
                        | AppEvent::Notify(msg) => {
                                self.set_status(msg);
                        },
                        | _ => {},
                }
        }

        pub fn set_status<S: Into<String>>(
                &mut self,
                msg: S,
        ) {
                self.status_msg = Some(msg.into());
        }

        pub fn start_qr_login(&mut self) {
                self.qr_status = QrLoginStatus::Generating;
                let tx = self.tx.clone();
                tokio::spawn(async move {
                        service::start_qr_login_flow(tx).await;
                });
        }

        pub fn logout(&mut self) {
                let _ = remove_user_info(APP_PATH.cache_auth_path());
                self.account = None;
                self.user_profile = None;
                self.qr_status = QrLoginStatus::Idle;
                self.set_status("已退出登录");
        }

        pub fn parse_current_url(&mut self) {
                if self.url_input.trim().is_empty() {
                        self.set_status("请输入有效视频链接或 BV 号");
                        return;
                }
                self.is_parsing = true;
                self.parse_error = None;
                self.set_status("正在解析视频信息...");
                let url = self.url_input.trim().to_string();
                let account = self.account.clone();
                let tx = self.tx.clone();
                tokio::spawn(async move {
                        service::parse_video_url(url, account, tx).await;
                });
        }

        pub fn start_download_current(&mut self) {
                if self.url_input.trim().is_empty() {
                        self.set_status("请先输入需要下载的链接或 BV 号");
                        return;
                }

                self.task_counter += 1;
                let task_id = self.task_counter;
                let url = self.url_input.trim().to_string();
                let mode = self.download_mode;
                let output_dir = PathBuf::from(&self.output_dir_input);
                let ffmpeg_path = if self.ffmpeg_path_input.trim().is_empty() {
                        None
                } else {
                        Some(PathBuf::from(&self.ffmpeg_path_input))
                };
                let video_quality = VIDEO_QUALITIES[self.video_quality_idx];
                let audio_quality = AUDIO_QUALITIES[self.audio_quality_idx];
                let video_encode = VIDEO_ENCODES[self.video_encode_idx];
                let batch = self.batch;

                let bvid = self
                        .preview_info
                        .as_ref()
                        .map(|p| p.bvid.clone())
                        .unwrap_or_else(|| url.clone());
                let title = self
                        .preview_info
                        .as_ref()
                        .map(|p| p.title.clone())
                        .unwrap_or_else(|| format!("任务-{}", task_id));

                let _ = self.tx.send(AppEvent::TaskCreated {
                        task_id,
                        bvid,
                        title,
                        mode,
                });

                let account = self.account.clone();

                let tx = self.tx.clone();
                tokio::spawn(async move {
                        service::execute_download(
                                task_id,
                                url,
                                mode,
                                output_dir,
                                ffmpeg_path,
                                video_quality,
                                audio_quality,
                                video_encode,
                                batch,
                                account,
                                tx,
                        )
                        .await;
                });

                self.tab = Tab::Tasks;
                self.set_status("已创建下载任务并转入后台执行");
        }

        pub fn save_settings(&mut self) {
                if let Ok(c) = self.concurrencies_input.parse::<usize>() {
                        if c > 0 {
                                self.config.concurrencies = c;
                        }
                }
                self.config.output_dir = if self.settings_output_input.trim().is_empty() {
                        None
                } else {
                        Some(self.settings_output_input.clone())
                };
                self.config.ffmpeg_path = if self.settings_ffmpeg_input.trim().is_empty() {
                        None
                } else {
                        Some(self.settings_ffmpeg_input.clone())
                };

                let _ = self.config.save(APP_PATH.config_path());
                self.output_dir_input = self.settings_output_input.clone();
                self.ffmpeg_path_input = self.settings_ffmpeg_input.clone();
                self.set_status("偏好设置已保存");
        }
}
