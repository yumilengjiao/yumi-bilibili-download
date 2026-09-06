//! Vim 交互与键盘事件调度模块
//!
//! 支持 Normal、Insert、Command 三种模式与极简 Vim 键位映射。

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
        app::App,
        model::{
                AUDIO_QUALITIES, DownloadFocus, DownloadMode, SettingsFocus, Tab, TaskStatus,
                VIDEO_ENCODES, VIDEO_QUALITIES, VimMode,
        },
};

pub fn handle_key_event(
        app: &mut App,
        key: KeyEvent,
) {
        if app.show_help {
                match key.code {
                        | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                                app.show_help = false;
                        },
                        | _ => {},
                }
                return;
        }

        match app.mode {
                | VimMode::Normal => handle_normal_mode(app, key),
                | VimMode::Insert => handle_insert_mode(app, key),
                | VimMode::Command => handle_command_mode(app, key),
        }
}

fn handle_normal_mode(
        app: &mut App,
        key: KeyEvent,
) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                        | KeyCode::Char('c') => {
                                app.should_quit = true;
                                return;
                        },
                        | _ => {},
                }
        }

        match key.code {
                | KeyCode::Char('q') => {
                        app.should_quit = true;
                },
                | KeyCode::Char(':') => {
                        app.mode = VimMode::Command;
                        app.command_input.clear();
                },
                | KeyCode::Char('?') => {
                        app.show_help = true;
                },
                | KeyCode::Char('1') => app.tab = Tab::Account,
                | KeyCode::Char('2') => app.tab = Tab::Download,
                | KeyCode::Char('3') => app.tab = Tab::Tasks,
                | KeyCode::Char('4') => app.tab = Tab::Settings,
                | KeyCode::Char('h') | KeyCode::Left => {
                        app.tab = app.tab.prev();
                },
                | KeyCode::Char('l') | KeyCode::Right => {
                        app.tab = app.tab.next();
                },
                | KeyCode::Tab => match app.tab {
                        | Tab::Download => {
                                app.download_focus = app.download_focus.next();
                        },
                        | Tab::Settings => {
                                app.settings_focus = app.settings_focus.next();
                        },
                        | _ => {
                                app.tab = app.tab.next();
                        },
                },
                | KeyCode::BackTab => match app.tab {
                        | Tab::Download => {
                                app.download_focus = app.download_focus.prev();
                        },
                        | Tab::Settings => {
                                app.settings_focus = app.settings_focus.prev();
                        },
                        | _ => {
                                app.tab = app.tab.prev();
                        },
                },
                | KeyCode::Char('j') | KeyCode::Down => match app.tab {
                        | Tab::Download => {
                                app.download_focus = app.download_focus.next();
                        },
                        | Tab::Tasks => {
                                if !app.tasks.is_empty()
                                        && app.selected_task_idx + 1 < app.tasks.len()
                                {
                                        app.selected_task_idx += 1;
                                }
                        },
                        | Tab::Settings => {
                                app.settings_focus = app.settings_focus.next();
                        },
                        | _ => {},
                },
                | KeyCode::Char('k') | KeyCode::Up => match app.tab {
                        | Tab::Download => {
                                app.download_focus = app.download_focus.prev();
                        },
                        | Tab::Tasks => {
                                if app.selected_task_idx > 0 {
                                        app.selected_task_idx -= 1;
                                }
                        },
                        | Tab::Settings => {
                                app.settings_focus = app.settings_focus.prev();
                        },
                        | _ => {},
                },
                | KeyCode::Char('i') | KeyCode::Char('a') => match app.tab {
                        | Tab::Download => match app.download_focus {
                                | DownloadFocus::UrlInput => {
                                        app.mode = VimMode::Insert;
                                        app.cursor_pos = app.url_input.len();
                                },
                                | DownloadFocus::OutputDir => {
                                        app.mode = VimMode::Insert;
                                        app.cursor_pos = app.output_dir_input.len();
                                },
                                | DownloadFocus::FfmpegPath => {
                                        app.mode = VimMode::Insert;
                                        app.cursor_pos = app.ffmpeg_path_input.len();
                                },
                                | _ => {},
                        },
                        | Tab::Settings => match app.settings_focus {
                                | SettingsFocus::Concurrencies => {
                                        app.mode = VimMode::Insert;
                                        app.cursor_pos = app.concurrencies_input.len();
                                },
                                | SettingsFocus::OutputDir => {
                                        app.mode = VimMode::Insert;
                                        app.cursor_pos = app.settings_output_input.len();
                                },
                                | SettingsFocus::FfmpegPath => {
                                        app.mode = VimMode::Insert;
                                        app.cursor_pos = app.settings_ffmpeg_input.len();
                                },
                        },
                        | _ => {},
                },
                | KeyCode::Char('r') => match app.tab {
                        | Tab::Account => {
                                app.start_qr_login();
                        },
                        | Tab::Download => {
                                app.parse_current_url();
                        },
                        | _ => {},
                },
                | KeyCode::Char('p') => match app.tab {
                        | Tab::Download => {
                                app.parse_current_url();
                        },
                        | _ => {},
                },
                | KeyCode::Char('s') => match app.tab {
                        | Tab::Download => {
                                app.batch = !app.batch;
                                app.set_status(if app.batch {
                                        "批量下载模式: 开启"
                                } else {
                                        "批量下载模式: 关闭"
                                });
                        },
                        | _ => {},
                },
                | KeyCode::Char(' ') => match app.tab {
                        | Tab::Download => match app.download_focus {
                                | DownloadFocus::ModeSelect => {
                                        app.download_mode = match app.download_mode {
                                                | DownloadMode::Video => DownloadMode::Audio,
                                                | DownloadMode::Audio => DownloadMode::Cover,
                                                | DownloadMode::Cover => DownloadMode::Video,
                                        };
                                },
                                | DownloadFocus::VideoQuality => {
                                        app.video_quality_idx =
                                                (app.video_quality_idx + 1) % VIDEO_QUALITIES.len();
                                },
                                | DownloadFocus::AudioQuality => {
                                        app.audio_quality_idx =
                                                (app.audio_quality_idx + 1) % AUDIO_QUALITIES.len();
                                },
                                | DownloadFocus::VideoEncode => {
                                        app.video_encode_idx =
                                                (app.video_encode_idx + 1) % VIDEO_ENCODES.len();
                                },
                                | DownloadFocus::BatchToggle => {
                                        app.batch = !app.batch;
                                },
                                | _ => {},
                        },
                        | _ => {},
                },
                | KeyCode::Char('c') => match app.tab {
                        | Tab::Tasks => {
                                app.tasks.retain(|t| t.status != TaskStatus::Completed);
                                app.set_status("已清空所有已完成任务");
                        },
                        | Tab::Download => match app.download_focus {
                                | DownloadFocus::UrlInput => {
                                        app.url_input.clear();
                                        app.cursor_pos = 0;
                                        app.preview_info = None;
                                },
                                | _ => {},
                        },
                        | _ => {},
                },
                | KeyCode::Char('d') => match app.tab {
                        | Tab::Tasks => {
                                if !app.tasks.is_empty() && app.selected_task_idx < app.tasks.len()
                                {
                                        let removed = app.tasks.remove(app.selected_task_idx);
                                        if app.selected_task_idx >= app.tasks.len()
                                                && app.selected_task_idx > 0
                                        {
                                                app.selected_task_idx -= 1;
                                        }
                                        app.set_status(format!("已移除任务: {}", removed.title));
                                }
                        },
                        | _ => {},
                },
                | KeyCode::Enter => match app.tab {
                        | Tab::Account => {
                                if app.account.is_none() {
                                        app.start_qr_login();
                                }
                        },
                        | Tab::Download => {
                                app.start_download_current();
                        },
                        | Tab::Settings => {
                                app.save_settings();
                        },
                        | _ => {},
                },
                | _ => {},
        }
}

fn handle_insert_mode(
        app: &mut App,
        key: KeyEvent,
) {
        match key.code {
                | KeyCode::Esc => {
                        app.mode = VimMode::Normal;
                },
                | KeyCode::Enter => {
                        app.mode = VimMode::Normal;
                        if app.tab == Tab::Download && app.download_focus == DownloadFocus::UrlInput
                        {
                                app.parse_current_url();
                        } else if app.tab == Tab::Settings {
                                app.save_settings();
                        }
                },
                | KeyCode::Left => {
                        if app.cursor_pos > 0 {
                                app.cursor_pos -= 1;
                        }
                },
                | KeyCode::Right => {
                        let len = get_active_input_len(app);
                        if app.cursor_pos < len {
                                app.cursor_pos += 1;
                        }
                },
                | KeyCode::Backspace => {
                        if app.cursor_pos > 0 {
                                let pos = app.cursor_pos - 1;
                                if let Some(input) = get_active_input_mut(app) {
                                        if pos < input.len() {
                                                input.remove(pos);
                                        }
                                }
                                app.cursor_pos = pos;
                        }
                },
                | KeyCode::Char(c) => {
                        let pos = app.cursor_pos;
                        if let Some(input) = get_active_input_mut(app) {
                                input.insert(pos, c);
                        }
                        app.cursor_pos = pos + 1;
                },
                | _ => {},
        }
}

fn handle_command_mode(
        app: &mut App,
        key: KeyEvent,
) {
        match key.code {
                | KeyCode::Esc => {
                        app.mode = VimMode::Normal;
                        app.command_input.clear();
                },
                | KeyCode::Backspace => {
                        if app.command_input.is_empty() {
                                app.mode = VimMode::Normal;
                        } else {
                                app.command_input.pop();
                        }
                },
                | KeyCode::Char(c) => {
                        app.command_input.push(c);
                },
                | KeyCode::Enter => {
                        let cmd = app.command_input.trim().to_string();
                        app.mode = VimMode::Normal;
                        app.command_input.clear();
                        execute_command(app, &cmd);
                },
                | _ => {},
        }
}

fn execute_command(
        app: &mut App,
        cmd: &str,
) {
        match cmd {
                | "q" | "quit" | "exit" => {
                        app.should_quit = true;
                },
                | "help" | "h" | "?" => {
                        app.show_help = true;
                },
                | "login" | "qr" => {
                        app.tab = Tab::Account;
                        app.start_qr_login();
                },
                | "logout" => {
                        app.logout();
                },
                | "w" | "write" | "save" => {
                        app.save_settings();
                },
                | "account" | "user" => {
                        app.tab = Tab::Account;
                },
                | "download" | "dl" => {
                        app.tab = Tab::Download;
                },
                | "tasks" | "task" => {
                        app.tab = Tab::Tasks;
                },
                | "settings" | "set" => {
                        app.tab = Tab::Settings;
                },
                | "clear" => match app.tab {
                        | Tab::Tasks => {
                                app.tasks.retain(|t| t.status != TaskStatus::Completed);
                                app.set_status("已清空所有已完成任务");
                        },
                        | Tab::Download => {
                                app.url_input.clear();
                                app.cursor_pos = 0;
                                app.preview_info = None;
                        },
                        | _ => {},
                },
                | "" => {},
                | other => {
                        app.set_status(format!("未知命令: :{}", other));
                },
        }
}

fn get_active_input_len(app: &App) -> usize {
        match app.tab {
                | Tab::Download => match app.download_focus {
                        | DownloadFocus::UrlInput => app.url_input.len(),
                        | DownloadFocus::OutputDir => app.output_dir_input.len(),
                        | DownloadFocus::FfmpegPath => app.ffmpeg_path_input.len(),
                        | _ => 0,
                },
                | Tab::Settings => match app.settings_focus {
                        | SettingsFocus::Concurrencies => app.concurrencies_input.len(),
                        | SettingsFocus::OutputDir => app.settings_output_input.len(),
                        | SettingsFocus::FfmpegPath => app.settings_ffmpeg_input.len(),
                },
                | _ => 0,
        }
}

fn get_active_input_mut(app: &mut App) -> Option<&mut String> {
        match app.tab {
                | Tab::Download => match app.download_focus {
                        | DownloadFocus::UrlInput => Some(&mut app.url_input),
                        | DownloadFocus::OutputDir => Some(&mut app.output_dir_input),
                        | DownloadFocus::FfmpegPath => Some(&mut app.ffmpeg_path_input),
                        | _ => None,
                },
                | Tab::Settings => match app.settings_focus {
                        | SettingsFocus::Concurrencies => Some(&mut app.concurrencies_input),
                        | SettingsFocus::OutputDir => Some(&mut app.settings_output_input),
                        | SettingsFocus::FfmpegPath => Some(&mut app.settings_ffmpeg_input),
                },
                | _ => None,
        }
}
