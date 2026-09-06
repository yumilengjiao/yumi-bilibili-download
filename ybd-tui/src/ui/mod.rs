//! TUI 视图渲染总路由与通用渲染辅助模块

pub mod account;
pub mod download;
pub mod header;
pub mod help;
pub mod settings;
pub mod status_bar;
pub mod tasks;

use ratatui::{
        Frame,
        layout::{Constraint, Direction, Layout},
};

use crate::{
        app::App,
        model::Tab,
        ui::{
                account::render_account_view, download::render_download_view,
                header::render_header, help::render_help_popup, settings::render_settings_view,
                status_bar::render_status_bar, tasks::render_tasks_view,
        },
};

pub fn render_ui(
        f: &mut Frame,
        app: &App,
) {
        let size = f.area();

        let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                        Constraint::Length(2), // 顶部导航
                        Constraint::Min(10),   // 主内容区
                        Constraint::Length(1), // 底部状态行
                ])
                .split(size);

        render_header(f, app, chunks[0]);

        match app.tab {
                Tab::Account => render_account_view(f, app, chunks[1]),
                Tab::Download => render_download_view(f, app, chunks[1]),
                Tab::Tasks => render_tasks_view(f, app, chunks[1]),
                Tab::Settings => render_settings_view(f, app, chunks[1]),
        }

        render_status_bar(f, app, chunks[2]);

        if app.show_help {
                render_help_popup(f, size);
        }
}

/// 计算单行输入框在有限宽度下的横向滚动切片与可视光标偏移量
///
/// 当输入长度超出 `max_width` 时，左侧内容自动向左滚动隐藏，保持光标与当前输入末尾始终可见。
pub fn get_scrolled_text(
        text: &str,
        cursor_pos: usize,
        max_width: usize,
) -> (String, u16) {
        let char_count = text.chars().count();
        if max_width == 0 {
                return (String::new(), 0);
        }

        if char_count <= max_width {
                return (text.to_string(), cursor_pos.min(char_count) as u16);
        }

        let clamped_cursor = cursor_pos.min(char_count);
        let start_char = if clamped_cursor >= max_width {
                clamped_cursor.saturating_sub(max_width.saturating_sub(1))
        } else {
                0
        };

        let visible_str: String = text.chars().skip(start_char).take(max_width).collect();
        let visual_cursor = (clamped_cursor - start_char) as u16;

        (visible_str, visual_cursor)
}
