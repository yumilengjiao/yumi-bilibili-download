//! 个人中心与扫码登录页面渲染模块

use ratatui::{
        Frame,
        layout::Rect,
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::Paragraph,
};

use crate::{app::App, model::QrLoginStatus};

pub fn render_account_view(
        f: &mut Frame,
        app: &App,
        area: Rect,
) {
        let mut lines = Vec::new();
        lines.push(Line::from(""));

        if let Some(ref account) = app.account {
                let status_color = if account.is_expired() {
                        Color::Red
                } else {
                        Color::Green
                };
                let status_text = if account.is_expired() {
                        "登录已过期 (请按 r 重新扫码登录)"
                } else {
                        "登录有效 (正常)"
                };

                lines.push(Line::from(vec![
                        Span::styled("  当前状态  ", Style::default().fg(Color::DarkGray)),
                        Span::styled("● ", Style::default().fg(status_color)),
                        Span::styled(
                                status_text,
                                Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD),
                        ),
                ]));
                lines.push(Line::from(""));

                lines.push(Line::from(vec![Span::styled(
                        "  账号资料",
                        Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                )]));

                let uname = app
                        .user_profile
                        .as_ref()
                        .map(|p| p.uname.as_str())
                        .unwrap_or("加载中...");
                let vip_type = app
                        .user_profile
                        .as_ref()
                        .map(|p| p.vip_type.as_str())
                        .unwrap_or("大会员");

                lines.push(Line::from(vec![
                        Span::styled("    用户昵称    ", Style::default().fg(Color::Gray)),
                        Span::styled(
                                uname,
                                Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD),
                        ),
                ]));

                lines.push(Line::from(vec![
                        Span::styled("    UID         ", Style::default().fg(Color::Gray)),
                        Span::styled(account.get_user_id(), Style::default().fg(Color::LightCyan)),
                ]));

                let sessdata = account.get_sessdata();
                let sess_display = if sessdata.len() > 10 {
                        format!("{}...{}", &sessdata[0..4], &sessdata[sessdata.len() - 4..])
                } else {
                        "******".to_string()
                };

                lines.push(Line::from(vec![
                        Span::styled("    通行凭据    ", Style::default().fg(Color::Gray)),
                        Span::styled(sess_display, Style::default().fg(Color::DarkGray)),
                        Span::styled(" (已脱敏)", Style::default().fg(Color::DarkGray)),
                ]));

                lines.push(Line::from(vec![
                        Span::styled("    会员权限    ", Style::default().fg(Color::Gray)),
                        Span::styled(
                                vip_type,
                                Style::default()
                                        .fg(Color::LightYellow)
                                        .add_modifier(Modifier::BOLD),
                        ),
                ]));

                lines.push(Line::from(vec![
                        Span::styled("    下载特权    ", Style::default().fg(Color::Gray)),
                        Span::styled(
                                "1080P60 / 4K / 8K 超清画质 & Hi-Res / 杜比无损音频",
                                Style::default().fg(Color::Green),
                        ),
                ]));

                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                        "  快捷操作",
                        Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                )]));
                lines.push(Line::from(vec![
                        Span::styled("    r ", Style::default().fg(Color::Cyan)),
                        Span::styled("重新扫码登录      ", Style::default().fg(Color::Gray)),
                        Span::styled(":logout ", Style::default().fg(Color::Cyan)),
                        Span::styled("退出登录", Style::default().fg(Color::Gray)),
                ]));
        } else {
                // 未登录状态
                lines.push(Line::from(vec![
                        Span::styled("  当前状态  ", Style::default().fg(Color::DarkGray)),
                        Span::styled("○ ", Style::default().fg(Color::Yellow)),
                        Span::styled(
                                "未登录 (按 r 启动扫码登录)",
                                Style::default()
                                        .fg(Color::Yellow)
                                        .add_modifier(Modifier::BOLD),
                        ),
                ]));
                lines.push(Line::from(""));

                match &app.qr_status {
                        | QrLoginStatus::Idle => {
                                lines.push(Line::from(vec![
                                        Span::styled("  提示: ", Style::default().fg(Color::Gray)),
                                        Span::styled(
                                                "登录后即可解锁 1080P60、4K 超高清画质及 Hi-Res 无损音频下载。",
                                                Style::default().fg(Color::DarkGray),
                                        ),
                                ]));
                                lines.push(Line::from(""));
                                lines.push(Line::from(vec![
                                        Span::styled("  操作: ", Style::default().fg(Color::Gray)),
                                        Span::styled(
                                                "按 [ r ] 或 [ Enter ] 开始生成登录二维码",
                                                Style::default().fg(Color::Cyan),
                                        ),
                                ]));
                        },
                        | QrLoginStatus::Generating => {
                                lines.push(Line::from(vec![Span::styled(
                                        "  正在请求并生成二维码...",
                                        Style::default().fg(Color::Yellow),
                                )]));
                        },
                        | QrLoginStatus::WaitingScan { qr_text, .. } => {
                                lines.push(Line::from(vec![Span::styled(
                                        "  请使用 哔哩哔哩 手机客户端 扫码登录：",
                                        Style::default().fg(Color::Cyan),
                                )]));
                                lines.push(Line::from(""));
                                for qr_line in qr_text.lines() {
                                        lines.push(Line::from(vec![
                                                Span::styled("    ", Style::default()),
                                                Span::styled(
                                                        qr_line.to_string(),
                                                        Style::default().fg(Color::White),
                                                ),
                                        ]));
                                }
                                lines.push(Line::from(""));
                                lines.push(Line::from(vec![
                                        Span::styled("  状态: ", Style::default().fg(Color::Gray)),
                                        Span::styled(
                                                "等待扫码中... (每 2 秒自动检查)",
                                                Style::default().fg(Color::LightYellow),
                                        ),
                                ]));
                        },
                        | QrLoginStatus::ScannedWaitingConfirm => {
                                lines.push(Line::from(vec![Span::styled(
                                        "  已扫码，请在手机端确认登录...",
                                        Style::default()
                                                .fg(Color::LightGreen)
                                                .add_modifier(Modifier::BOLD),
                                )]));
                        },
                        | QrLoginStatus::Success => {
                                lines.push(Line::from(vec![Span::styled(
                                        "  ✓ 登录成功！",
                                        Style::default()
                                                .fg(Color::Green)
                                                .add_modifier(Modifier::BOLD),
                                )]));
                        },
                        | QrLoginStatus::Expired => {
                                lines.push(Line::from(vec![Span::styled(
                                        "  ✗ 二维码已过期，按 [ r ] 重新生成",
                                        Style::default().fg(Color::Red),
                                )]));
                        },
                        | QrLoginStatus::Error(e) => {
                                lines.push(Line::from(vec![Span::styled(
                                        format!("  ✗ 登录异常: {}", e),
                                        Style::default().fg(Color::Red),
                                )]));
                        },
                }
        }

        f.render_widget(Paragraph::new(lines), area);
}
