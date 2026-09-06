//! 顶部极简导航栏渲染模块

use ratatui::{
        Frame,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::Paragraph,
};

use crate::{app::App, model::Tab};

pub fn render_header(
        f: &mut Frame,
        app: &App,
        area: Rect,
) {
        let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(20), Constraint::Min(40)])
                .split(area);

        // 左侧 Logo
        let logo = Line::from(vec![
                Span::styled(
                        " ybd-tui ",
                        Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                ),
                Span::styled("v0.2.2", Style::default().fg(Color::DarkGray)),
        ]);
        f.render_widget(Paragraph::new(logo), chunks[0]);

        // 右侧无框极简 Tab
        let tabs = [
                (Tab::Account, "个人中心"),
                (Tab::Download, "资源下载"),
                (Tab::Tasks, "任务列表"),
                (Tab::Settings, "偏好设置"),
        ];

        let mut spans = Vec::new();
        for (i, (t, name)) in tabs.iter().enumerate() {
                if i > 0 {
                        spans.push(Span::styled("   ", Style::default().fg(Color::DarkGray)));
                }
                let is_active = app.tab == *t;
                if is_active {
                        spans.push(Span::styled(
                                "● ",
                                Style::default()
                                        .fg(Color::Cyan)
                                        .add_modifier(Modifier::BOLD),
                        ));
                        spans.push(Span::styled(
                                *name,
                                Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD),
                        ));
                } else {
                        spans.push(Span::styled("○ ", Style::default().fg(Color::DarkGray)));
                        spans.push(Span::styled(*name, Style::default().fg(Color::DarkGray)));
                }
        }

        let tab_line = Line::from(spans);
        f.render_widget(Paragraph::new(tab_line), chunks[1]);
}
