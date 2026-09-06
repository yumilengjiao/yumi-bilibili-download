//! Vim 交互与键盘事件调度模块
//!
//! 按页面（个人中心、资源下载、任务列表、偏好设置）解耦处理快捷键，支持 Emacs 风格行编辑。

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
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                                app.show_help = false;
                        },
                        _ => {},
                }
                return;
        }

        match app.mode {
                VimMode::Normal => handle_normal_mode(app, key),
                VimMode::Insert => handle_insert_mode(app, key),
                VimMode::Command => handle_command_mode(app, key),
        }
}

fn handle_normal_mode(
        app: &mut App,
        key: KeyEvent,
) {
        // 1. 全局退出快捷键 (Ctrl+C / q)
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                app.should_quit = true;
                return;
        }

        // 2. 全局基础导航与模式切换
        match key.code {
                KeyCode::Char('q') => {
                        app.should_quit = true;
                        return;
                },
                KeyCode::Char(':') => {
                        app.mode = VimMode::Command;
                        app.command_input.clear();
                        return;
                },
                KeyCode::Char('?') => {
                        app.show_help = true;
                        return;
                },
                KeyCode::Char('1') => {
                        app.tab = Tab::Account;
                        app.status_msg = None;
                        return;
                },
                KeyCode::Char('2') => {
                        app.tab = Tab::Download;
                        app.status_msg = None;
                        return;
                },
                KeyCode::Char('3') => {
                        app.tab = Tab::Tasks;
                        app.status_msg = None;
                        return;
                },
                KeyCode::Char('4') => {
                        app.tab = Tab::Settings;
                        app.status_msg = None;
                        return;
                },
                KeyCode::Char('h') | KeyCode::Left => {
                        app.tab = app.tab.prev();
                        app.status_msg = None;
                        return;
                },
                KeyCode::Char('l') | KeyCode::Right => {
                        app.tab = app.tab.next();
                        app.status_msg = None;
                        return;
                },
                _ => {},
        }

        // 3. 各页面专属按键分发
        match app.tab {
                Tab::Account => handle_account_tab_keys(app, key),
                Tab::Download => handle_download_tab_keys(app, key),
                Tab::Tasks => handle_tasks_tab_keys(app, key),
                Tab::Settings => handle_settings_tab_keys(app, key),
        }
}

/// 👤 个人中心页面按键处理
fn handle_account_tab_keys(
        app: &mut App,
        key: KeyEvent,
) {
        match key.code {
                KeyCode::Char('r') => {
                        app.start_qr_login();
                },
                KeyCode::Enter => {
                        if app.account.is_none() {
                                app.start_qr_login();
                        }
                },
                _ => {},
        }
}

/// 📥 资源下载页面按键处理
fn handle_download_tab_keys(
        app: &mut App,
        key: KeyEvent,
) {
        match key.code {
                KeyCode::Tab | KeyCode::Char('j') | KeyCode::Down => {
                        app.download_focus = app.download_focus.next();
                },
                KeyCode::BackTab | KeyCode::Char('k') | KeyCode::Up => {
                        app.download_focus = app.download_focus.prev();
                },
                KeyCode::Char('i') | KeyCode::Char('a') => match app.download_focus {
                        DownloadFocus::UrlInput => {
                                app.mode = VimMode::Insert;
                                app.cursor_pos = app.url_input.len();
                        },
                        DownloadFocus::OutputDir => {
                                app.mode = VimMode::Insert;
                                app.cursor_pos = app.output_dir_input.len();
                        },
                        DownloadFocus::FfmpegPath => {
                                app.mode = VimMode::Insert;
                                app.cursor_pos = app.ffmpeg_path_input.len();
                        },
                        _ => {},
                },
                KeyCode::Char('p') | KeyCode::Char('r') => {
                        app.parse_current_url();
                },
                KeyCode::Char('s') => {
                        app.batch = !app.batch;
                        app.set_status(if app.batch {
                                "批量下载模式: 开启"
                        } else {
                                "批量下载模式: 关闭"
                        });
                },
                KeyCode::Char(' ') => match app.download_focus {
                        DownloadFocus::ModeSelect => {
                                app.download_mode = match app.download_mode {
                                        DownloadMode::Video => DownloadMode::Audio,
                                        DownloadMode::Audio => DownloadMode::Cover,
                                        DownloadMode::Cover => DownloadMode::Video,
                                };
                        },
                        DownloadFocus::VideoQuality => {
                                app.video_quality_idx =
                                        (app.video_quality_idx + 1) % VIDEO_QUALITIES.len();
                        },
                        DownloadFocus::AudioQuality => {
                                app.audio_quality_idx =
                                        (app.audio_quality_idx + 1) % AUDIO_QUALITIES.len();
                        },
                        DownloadFocus::VideoEncode => {
                                app.video_encode_idx =
                                        (app.video_encode_idx + 1) % VIDEO_ENCODES.len();
                        },
                        DownloadFocus::BatchToggle => {
                                app.batch = !app.batch;
                        },
                        _ => {},
                },
                KeyCode::Char('c') => {
                        if app.download_focus == DownloadFocus::UrlInput {
                                app.url_input.clear();
                                app.cursor_pos = 0;
                                app.preview_info = None;
                        }
                },
                KeyCode::Enter => {
                        app.start_download_current();
                },
                _ => {},
        }
}

/// 📊 任务列表页面按键处理
fn handle_tasks_tab_keys(
        app: &mut App,
        key: KeyEvent,
) {
        match key.code {
                KeyCode::Char('j') | KeyCode::Down => {
                        let active_count = app
                                .tasks
                                .iter()
                                .filter(|t| t.status != TaskStatus::Completed)
                                .count();
                        if active_count > 0 && app.selected_task_idx + 1 < active_count {
                                app.selected_task_idx += 1;
                        }
                },
                KeyCode::Char('k') | KeyCode::Up => {
                        if app.selected_task_idx > 0 {
                                app.selected_task_idx -= 1;
                        }
                },
                KeyCode::Char('c') => {
                        app.tasks.retain(|t| t.status != TaskStatus::Completed);
                        app.set_status("已清空所有已完成任务");
                },
                KeyCode::Char('d') => {
                        let target_id = app
                                .tasks
                                .iter()
                                .filter(|t| t.status != TaskStatus::Completed)
                                .nth(app.selected_task_idx)
                                .map(|t| t.id);

                        if let Some(id) = target_id {
                                if let Some(pos) = app.tasks.iter().position(|t| t.id == id) {
                                        let removed = app.tasks.remove(pos);
                                        let active_count = app
                                                .tasks
                                                .iter()
                                                .filter(|t| t.status != TaskStatus::Completed)
                                                .count();
                                        if app.selected_task_idx >= active_count
                                                && app.selected_task_idx > 0
                                        {
                                                app.selected_task_idx -= 1;
                                        }
                                        app.set_status(format!("已移除任务: {}", removed.title));
                                }
                        }
                },
                _ => {},
        }
}

/// ⚙️ 偏好设置页面按键处理
fn handle_settings_tab_keys(
        app: &mut App,
        key: KeyEvent,
) {
        match key.code {
                KeyCode::Tab | KeyCode::Char('j') | KeyCode::Down => {
                        app.settings_focus = app.settings_focus.next();
                },
                KeyCode::BackTab | KeyCode::Char('k') | KeyCode::Up => {
                        app.settings_focus = app.settings_focus.prev();
                },
                KeyCode::Char('i') | KeyCode::Char('a') => match app.settings_focus {
                        SettingsFocus::Concurrencies => {
                                app.mode = VimMode::Insert;
                                app.cursor_pos = app.concurrencies_input.len();
                        },
                        SettingsFocus::OutputDir => {
                                app.mode = VimMode::Insert;
                                app.cursor_pos = app.settings_output_input.len();
                        },
                        SettingsFocus::FfmpegPath => {
                                app.mode = VimMode::Insert;
                                app.cursor_pos = app.settings_ffmpeg_input.len();
                        },
                },
                _ => {},
        }
}

/// ✍️ 插入编辑模式处理 (支持完整 Emacs / Readline 快捷键)
fn handle_insert_mode(
        app: &mut App,
        key: KeyEvent,
) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                        KeyCode::Char('a') => {
                                // Ctrl+A: 移动到行首
                                app.cursor_pos = 0;
                                return;
                        },
                        KeyCode::Char('e') => {
                                // Ctrl+E: 移动到行尾
                                app.cursor_pos = get_active_input_len(app);
                                return;
                        },
                        KeyCode::Char('u') => {
                                // Ctrl+U: 清空整行
                                if let Some(input) = get_active_input_mut(app) {
                                        input.clear();
                                        app.cursor_pos = 0;
                                }
                                return;
                        },
                        KeyCode::Char('k') => {
                                // Ctrl+K: 删除光标至行尾内容
                                let pos = app.cursor_pos;
                                if let Some(input) = get_active_input_mut(app) {
                                        if pos < input.len() {
                                                input.truncate(pos);
                                        }
                                }
                                return;
                        },
                        KeyCode::Char('w') => {
                                // Ctrl+W: 向前删除一个单词
                                let pos = app.cursor_pos;
                                if pos > 0 {
                                        if let Some(input) = get_active_input_mut(app) {
                                                let prefix = &input[..pos];
                                                let trimmed = prefix.trim_end();
                                                let new_pos = match trimmed.rfind(|c: char| {
                                                        c.is_whitespace() || c == '/' || c == '?'
                                                }) {
                                                        Some(idx) => idx + 1,
                                                        None => 0,
                                                };
                                                input.replace_range(new_pos..pos, "");
                                                app.cursor_pos = new_pos;
                                        }
                                }
                                return;
                        },
                        KeyCode::Char('d') => {
                                // Ctrl+D: 删除光标处的字符
                                let pos = app.cursor_pos;
                                if let Some(input) = get_active_input_mut(app) {
                                        if pos < input.len() {
                                                input.remove(pos);
                                        }
                                }
                                return;
                        },
                        KeyCode::Char('h') => {
                                // Ctrl+H: 向前退格删除
                                if app.cursor_pos > 0 {
                                        let pos = app.cursor_pos - 1;
                                        if let Some(input) = get_active_input_mut(app) {
                                                if pos < input.len() {
                                                        input.remove(pos);
                                                }
                                        }
                                        app.cursor_pos = pos;
                                }
                                return;
                        },
                        KeyCode::Char('b') => {
                                // Ctrl+B: 向左移动一个字符
                                if app.cursor_pos > 0 {
                                        app.cursor_pos -= 1;
                                }
                                return;
                        },
                        KeyCode::Char('f') => {
                                // Ctrl+F: 向右移动一个字符
                                let len = get_active_input_len(app);
                                if app.cursor_pos < len {
                                        app.cursor_pos += 1;
                                }
                                return;
                        },
                        KeyCode::Char('c') => {
                                // Ctrl+C: 退出编辑模式
                                app.mode = VimMode::Normal;
                                return;
                        },
                        _ => {},
                }
        }

        match key.code {
                KeyCode::Esc => {
                        app.mode = VimMode::Normal;
                },
                KeyCode::Home => {
                        app.cursor_pos = 0;
                },
                KeyCode::End => {
                        app.cursor_pos = get_active_input_len(app);
                },
                KeyCode::Enter => {
                        app.mode = VimMode::Normal;
                        if app.tab == Tab::Download && app.download_focus == DownloadFocus::UrlInput
                        {
                                app.parse_current_url();
                        }
                },
                KeyCode::Left => {
                        if app.cursor_pos > 0 {
                                app.cursor_pos -= 1;
                        }
                },
                KeyCode::Right => {
                        let len = get_active_input_len(app);
                        if app.cursor_pos < len {
                                app.cursor_pos += 1;
                        }
                },
                KeyCode::Backspace => {
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
                KeyCode::Delete => {
                        let pos = app.cursor_pos;
                        if let Some(input) = get_active_input_mut(app) {
                                if pos < input.len() {
                                        input.remove(pos);
                                }
                        }
                },
                KeyCode::Char(c) => {
                        let pos = app.cursor_pos;
                        if let Some(input) = get_active_input_mut(app) {
                                input.insert(pos, c);
                        }
                        app.cursor_pos = pos + 1;
                },
                _ => {},
        }
}

/// ⌨️ 底部命令行模式处理 (:cmd)
fn handle_command_mode(
        app: &mut App,
        key: KeyEvent,
) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                        KeyCode::Char('u') | KeyCode::Char('c') => {
                                app.mode = VimMode::Normal;
                                app.command_input.clear();
                                return;
                        },
                        KeyCode::Char('a') => {
                                return;
                        },
                        KeyCode::Char('h') => {
                                app.command_input.pop();
                                return;
                        },
                        _ => {},
                }
        }

        match key.code {
                KeyCode::Esc => {
                        app.mode = VimMode::Normal;
                        app.command_input.clear();
                },
                KeyCode::Backspace => {
                        if app.command_input.is_empty() {
                                app.mode = VimMode::Normal;
                        } else {
                                app.command_input.pop();
                        }
                },
                KeyCode::Char(c) => {
                        app.command_input.push(c);
                },
                KeyCode::Enter => {
                        let cmd = app.command_input.trim().to_string();
                        app.mode = VimMode::Normal;
                        app.command_input.clear();
                        execute_command(app, &cmd);
                },
                _ => {},
        }
}

fn execute_command(
        app: &mut App,
        cmd: &str,
) {
        match cmd {
                "q" | "quit" | "exit" => {
                        app.should_quit = true;
                },
                "help" | "h" | "?" => {
                        app.show_help = true;
                },
                "login" | "qr" => {
                        app.tab = Tab::Account;
                        app.start_qr_login();
                },
                "logout" => {
                        app.logout();
                },
                "w" | "write" | "save" => {
                        app.save_settings();
                },
                "account" | "user" => {
                        app.tab = Tab::Account;
                },
                "download" | "dl" => {
                        app.tab = Tab::Download;
                },
                "tasks" | "task" => {
                        app.tab = Tab::Tasks;
                },
                "settings" | "set" => {
                        app.tab = Tab::Settings;
                },
                "clear" => match app.tab {
                        Tab::Tasks => {
                                app.tasks.retain(|t| t.status != TaskStatus::Completed);
                                app.set_status("已清空所有已完成任务");
                        },
                        Tab::Download => {
                                app.url_input.clear();
                                app.cursor_pos = 0;
                                app.preview_info = None;
                        },
                        _ => {},
                },
                "" => {},
                other => {
                        app.set_status(format!("未知命令: :{}", other));
                },
        }
}

fn get_active_input_len(app: &App) -> usize {
        match app.tab {
                Tab::Download => match app.download_focus {
                        DownloadFocus::UrlInput => app.url_input.len(),
                        DownloadFocus::OutputDir => app.output_dir_input.len(),
                        DownloadFocus::FfmpegPath => app.ffmpeg_path_input.len(),
                        _ => 0,
                },
                Tab::Settings => match app.settings_focus {
                        SettingsFocus::Concurrencies => app.concurrencies_input.len(),
                        SettingsFocus::OutputDir => app.settings_output_input.len(),
                        SettingsFocus::FfmpegPath => app.settings_ffmpeg_input.len(),
                },
                _ => 0,
        }
}

fn get_active_input_mut(app: &mut App) -> Option<&mut String> {
        match app.tab {
                Tab::Download => match app.download_focus {
                        DownloadFocus::UrlInput => Some(&mut app.url_input),
                        DownloadFocus::OutputDir => Some(&mut app.output_dir_input),
                        DownloadFocus::FfmpegPath => Some(&mut app.ffmpeg_path_input),
                        _ => None,
                },
                Tab::Settings => match app.settings_focus {
                        SettingsFocus::Concurrencies => Some(&mut app.concurrencies_input),
                        SettingsFocus::OutputDir => Some(&mut app.settings_output_input),
                        SettingsFocus::FfmpegPath => Some(&mut app.settings_ffmpeg_input),
                },
                _ => None,
        }
}
