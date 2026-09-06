//! 底部极简 Vim 状态栏与命令行渲染模块

use ratatui::{
        Frame,
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::Paragraph,
};

use crate::{app::App, model::VimMode};

pub fn render_status_bar(
        f: &mut Frame,
        app: &App,
        area: Rect,
) {
        let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(20), Constraint::Length(30)])
                .split(area);

        match app.mode {
                VimMode::Command => {
                        let cmd_line = Line::from(vec![
                                Span::styled(":", Style::default().fg(Color::Yellow)),
                                Span::styled(
                                        &app.command_input,
                                        Style::default()
                                                .fg(Color::White)
                                                .add_modifier(Modifier::BOLD),
                                ),
                        ]);
                        f.render_widget(Paragraph::new(cmd_line), chunks[0]);
                        let x = chunks[0].x + 1 + app.command_input.len() as u16;
                        let y = chunks[0].y;
                        f.set_cursor_position((x, y));
                },
                VimMode::Insert => {
                        let msg_span = if let Some(ref msg) = app.status_msg {
                                Span::styled(
                                        format!("  {}", msg),
                                        Style::default().fg(Color::DarkGray),
                                )
                        } else {
                                Span::styled(
                                        "  [Esc] 完成输入",
                                        Style::default().fg(Color::DarkGray),
                                )
                        };
                        f.render_widget(Paragraph::new(Line::from(vec![msg_span])), chunks[0]);
                },
                VimMode::Normal => {
                        let msg_span = if let Some(ref msg) = app.status_msg {
                                Span::styled(
                                        format!("  {}", msg),
                                        Style::default().fg(Color::LightYellow),
                                )
                        } else {
                                Span::raw("")
                        };
                        f.render_widget(Paragraph::new(Line::from(vec![msg_span])), chunks[0]);
                },
        }

        // 右侧极简帮助提示
        let help_text = match app.mode {
                VimMode::Command => Line::from(Span::styled(
                        "[Enter] 执行  [Esc] 取消",
                        Style::default().fg(Color::DarkGray),
                )),
                VimMode::Insert => Line::from(Span::styled(
                        "[Enter] 确认  [Esc] 退出",
                        Style::default().fg(Color::DarkGray),
                )),
                VimMode::Normal => Line::from(Span::styled(
                        ":help 或 ? 查看帮助",
                        Style::default().fg(Color::DarkGray),
                )),
        };

        f.render_widget(
                Paragraph::new(help_text).alignment(Alignment::Right),
                chunks[1],
        );
}
