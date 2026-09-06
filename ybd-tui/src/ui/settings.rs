//! 偏好设置视图渲染模块

use ratatui::{
        Frame,
        layout::Rect,
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::Paragraph,
};

use crate::{
        app::App,
        model::{SettingsFocus, VimMode},
        ui::get_scrolled_text,
};

pub fn render_settings_view(
        f: &mut Frame,
        app: &App,
        area: Rect,
) {
        let mut lines = Vec::new();
        lines.push(Line::from(""));

        lines.push(Line::from(vec![
                Span::styled(
                        "  偏好设置",
                        Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                        " (按 i 编辑，按 Enter / :w 保存配置)",
                        Style::default().fg(Color::DarkGray),
                ),
        ]));
        lines.push(Line::from(""));

        let available_width = area.width.saturating_sub(22) as usize;

        // 并发数
        let is_c_focus = app.settings_focus == SettingsFocus::Concurrencies;
        let c_display_line_idx = lines.len();
        let (scrolled_c, visual_c_cursor) =
                get_scrolled_text(&app.concurrencies_input, app.cursor_pos, available_width);

        lines.push(Line::from(vec![
                Span::styled(
                        if is_c_focus {
                                "  ❯ 最大下载并发数  "
                        } else {
                                "    最大下载并发数  "
                        },
                        if is_c_focus {
                                Style::default().fg(Color::Cyan)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
                Span::styled(
                        scrolled_c,
                        if is_c_focus {
                                Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
        ]));

        // 默认输出目录
        let is_out_focus = app.settings_focus == SettingsFocus::OutputDir;
        let out_display_line_idx = lines.len();
        let (scrolled_out, visual_out_cursor) =
                get_scrolled_text(&app.settings_output_input, app.cursor_pos, available_width);

        lines.push(Line::from(vec![
                Span::styled(
                        if is_out_focus {
                                "  ❯ 默认保存目录    "
                        } else {
                                "    默认保存目录    "
                        },
                        if is_out_focus {
                                Style::default().fg(Color::Cyan)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
                Span::styled(
                        scrolled_out,
                        if is_out_focus {
                                Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD)
                        } else {
                                Style::default().fg(Color::DarkGray)
                        },
                ),
        ]));

        // 默认 FFmpeg 路径
        let is_ff_focus = app.settings_focus == SettingsFocus::FfmpegPath;
        let ff_display_line_idx = lines.len();
        let (scrolled_ff, visual_ff_cursor) =
                get_scrolled_text(&app.settings_ffmpeg_input, app.cursor_pos, available_width);

        let ff_display = if app.settings_ffmpeg_input.is_empty() && !is_ff_focus {
                "(留空则自动从 PATH 读取 ffmpeg)".to_string()
        } else {
                scrolled_ff
        };
        lines.push(Line::from(vec![
                Span::styled(
                        if is_ff_focus {
                                "  ❯ FFmpeg 执行路径 "
                        } else {
                                "    FFmpeg 执行路径 "
                        },
                        if is_ff_focus {
                                Style::default().fg(Color::Cyan)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
                Span::styled(
                        ff_display,
                        if is_ff_focus {
                                Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD)
                        } else {
                                Style::default().fg(Color::DarkGray)
                        },
                ),
        ]));

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
                Span::styled("  快捷操作: ", Style::default().fg(Color::DarkGray)),
                Span::styled("i ", Style::default().fg(Color::Cyan)),
                Span::styled("修改输入     ", Style::default().fg(Color::Gray)),
                Span::styled("Enter / :w ", Style::default().fg(Color::Cyan)),
                Span::styled("保存设置", Style::default().fg(Color::Gray)),
        ]));

        f.render_widget(Paragraph::new(lines), area);

        // 设置终端真实光标位置（结合横向滚动的视觉偏移）
        if app.mode == VimMode::Insert {
                match app.settings_focus {
                        | SettingsFocus::Concurrencies => {
                                let x = area.x + 20 + visual_c_cursor;
                                let y = area.y + c_display_line_idx as u16;
                                f.set_cursor_position((x, y));
                        },
                        | SettingsFocus::OutputDir => {
                                let x = area.x + 20 + visual_out_cursor;
                                let y = area.y + out_display_line_idx as u16;
                                f.set_cursor_position((x, y));
                        },
                        | SettingsFocus::FfmpegPath => {
                                let x = area.x + 20 + visual_ff_cursor;
                                let y = area.y + ff_display_line_idx as u16;
                                f.set_cursor_position((x, y));
                        },
                }
        }
}
