//! 帮助弹窗浮层模块

use ratatui::{
        Frame,
        layout::Rect,
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

pub fn render_help_popup(
        f: &mut Frame,
        area: Rect,
) {
        let popup_width = 70.min(area.width.saturating_sub(4));
        let popup_height = 22.min(area.height.saturating_sub(2));

        let x = (area.width.saturating_sub(popup_width)) / 2;
        let y = (area.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(x, y, popup_width, popup_height);

        f.render_widget(Clear, popup_area);

        let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(
                        " 帮助与快捷键指南 ",
                        Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                ));

        let text = vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                        "  导航与切换",
                        Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![
                        Span::styled("    1-4 / h / l         ", Style::default().fg(Color::Cyan)),
                        Span::styled("切换顶部标签页", Style::default().fg(Color::Gray)),
                ]),
                Line::from(vec![
                        Span::styled("    Tab / BackTab       ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                                "在当前页面的输入/选项间循环切换",
                                Style::default().fg(Color::Gray),
                        ),
                ]),
                Line::from(vec![
                        Span::styled("    j / k               ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                                "向下 / 向上移动焦点或选中列表项",
                                Style::default().fg(Color::Gray),
                        ),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                        "  模式切换",
                        Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![
                        Span::styled("    i / a               ", Style::default().fg(Color::Cyan)),
                        Span::styled("进入文本输入编辑模式", Style::default().fg(Color::Gray)),
                ]),
                Line::from(vec![
                        Span::styled("    Esc                 ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                                "退出编辑，返回正常浏览模式",
                                Style::default().fg(Color::Gray),
                        ),
                ]),
                Line::from(vec![
                        Span::styled("    :                   ", Style::default().fg(Color::Cyan)),
                        Span::styled("进入底部命令行模式", Style::default().fg(Color::Gray)),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                        "  常用操作",
                        Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![
                        Span::styled("    r                   ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                                "扫码登录 (个人中心) / 解析视频 (下载页)",
                                Style::default().fg(Color::Gray),
                        ),
                ]),
                Line::from(vec![
                        Span::styled("    p                   ", Style::default().fg(Color::Cyan)),
                        Span::styled("解析当前输入的视频链接", Style::default().fg(Color::Gray)),
                ]),
                Line::from(vec![
                        Span::styled("    Space               ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                                "切换单选/下拉选项 (画质/音质/编码/模式)",
                                Style::default().fg(Color::Gray),
                        ),
                ]),
                Line::from(vec![
                        Span::styled("    Enter               ", Style::default().fg(Color::Cyan)),
                        Span::styled("开始下载 / 确认保存", Style::default().fg(Color::Gray)),
                ]),
                Line::from(vec![
                        Span::styled("    c / d               ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                                "清空输入 / 清除已完成任务 / 移除任务",
                                Style::default().fg(Color::Gray),
                        ),
                ]),
                Line::from(vec![
                        Span::styled("    q                   ", Style::default().fg(Color::Cyan)),
                        Span::styled("退出程序", Style::default().fg(Color::Gray)),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                        "  按 [Esc] 或 [q] 关闭帮助窗口",
                        Style::default().fg(Color::DarkGray),
                )]),
        ];

        let p = Paragraph::new(text).block(block);
        f.render_widget(p, popup_area);
}
