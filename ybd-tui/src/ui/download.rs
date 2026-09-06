//! 资源解析与下载配置视图

use ratatui::{
        Frame,
        layout::Rect,
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::Paragraph,
};

use crate::{
        app::App,
        model::{
                AUDIO_QUALITIES, DownloadFocus, DownloadMode, VIDEO_ENCODES, VIDEO_QUALITIES,
                VimMode,
        },
        ui::get_scrolled_text,
};

pub fn render_download_view(
        f: &mut Frame,
        app: &App,
        area: Rect,
) {
        let mut lines = Vec::new();
        lines.push(Line::from(""));

        // 目标链接输入框（可用宽度计算）
        let is_url_focus = app.download_focus == DownloadFocus::UrlInput;
        let prefix = if is_url_focus { "❯ " } else { "  " };
        lines.push(Line::from(vec![
                Span::styled(
                        "  目标链接",
                        Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                ),
                if is_url_focus {
                        Span::styled(
                                " (按 i 输入 / Enter 解析)",
                                Style::default().fg(Color::DarkGray),
                        )
                } else {
                        Span::raw("")
                },
        ]));

        let available_width = area.width.saturating_sub(6) as usize;
        let (scrolled_url, visual_url_cursor) =
                get_scrolled_text(&app.url_input, app.cursor_pos, available_width);

        let url_display = if app.url_input.is_empty() && !is_url_focus {
                "https://www.bilibili.com/video/BV... 或 BV号 / 合集链接".to_string()
        } else {
                scrolled_url
        };

        lines.push(Line::from(vec![
                Span::styled(
                        format!("  {}", prefix),
                        Style::default().fg(if is_url_focus {
                                Color::Cyan
                        } else {
                                Color::DarkGray
                        }),
                ),
                Span::styled(
                        url_display,
                        if is_url_focus {
                                Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD)
                        } else {
                                Style::default().fg(Color::DarkGray)
                        },
                ),
        ]));
        lines.push(Line::from(""));

        // 视频预览信息
        if let Some(ref info) = app.preview_info {
                lines.push(Line::from(vec![Span::styled(
                        "  视频信息",
                        Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                )]));
                lines.push(Line::from(vec![
                        Span::styled("    标题    ", Style::default().fg(Color::Gray)),
                        Span::styled(
                                &info.title,
                                Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD),
                        ),
                ]));
                if info.is_collection {
                        let count_str = info
                                .media_count
                                .map(|c| format!("共 {} 个视频", c))
                                .unwrap_or_else(|| "合集".to_string());
                        lines.push(Line::from(vec![
                                Span::styled("    类型    ", Style::default().fg(Color::Gray)),
                                Span::styled(
                                        format!("收藏夹 / 媒体合集 ({})", count_str),
                                        Style::default()
                                                .fg(Color::LightYellow)
                                                .add_modifier(Modifier::BOLD),
                                ),
                        ]));
                        lines.push(Line::from(vec![
                                Span::styled("    创建者  ", Style::default().fg(Color::Gray)),
                                Span::styled(&info.owner, Style::default().fg(Color::LightCyan)),
                        ]));
                        lines.push(Line::from(vec![
                                Span::styled("    合集ID  ", Style::default().fg(Color::Gray)),
                                Span::styled(&info.bvid, Style::default().fg(Color::DarkGray)),
                        ]));
                } else {
                        lines.push(Line::from(vec![
                                Span::styled("    UP主    ", Style::default().fg(Color::Gray)),
                                Span::styled(
                                        format!(
                                                "{}  ({}:{:02})",
                                                info.owner,
                                                info.duration_secs / 60,
                                                info.duration_secs % 60
                                        ),
                                        Style::default().fg(Color::LightCyan),
                                ),
                        ]));
                        lines.push(Line::from(vec![
                                Span::styled("    BV号    ", Style::default().fg(Color::Gray)),
                                Span::styled(&info.bvid, Style::default().fg(Color::DarkGray)),
                        ]));
                }
                if !info.desc.is_empty() {
                        let short_desc = if info.desc.len() > 60 {
                                format!("{}...", &info.desc[0..60])
                        } else {
                                info.desc.clone()
                        };
                        lines.push(Line::from(vec![
                                Span::styled("    描述    ", Style::default().fg(Color::Gray)),
                                Span::styled(short_desc, Style::default().fg(Color::DarkGray)),
                        ]));
                }
                lines.push(Line::from(""));
        } else if app.is_parsing {
                lines.push(Line::from(vec![Span::styled(
                        "  正在解析视频信息...",
                        Style::default().fg(Color::Yellow),
                )]));
                lines.push(Line::from(""));
        }

        // 下载参数
        lines.push(Line::from(vec![
                Span::styled(
                        "  下载参数",
                        Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                        " (Tab 切换 / Space 选择)",
                        Style::default().fg(Color::DarkGray),
                ),
        ]));

        // 模式
        let is_mode_focus = app.download_focus == DownloadFocus::ModeSelect;
        let mode_v = match app.download_mode {
                | DownloadMode::Video => "● 视频     ○ 仅音频     ○ 仅封面",
                | DownloadMode::Audio => "○ 视频     ● 仅音频     ○ 仅封面",
                | DownloadMode::Cover => "○ 视频     ○ 仅音频     ● 仅封面",
        };
        lines.push(Line::from(vec![
                Span::styled(
                        if is_mode_focus {
                                "  ❯ 模式    "
                        } else {
                                "    模式    "
                        },
                        if is_mode_focus {
                                Style::default().fg(Color::Cyan)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
                Span::styled(
                        mode_v,
                        if is_mode_focus {
                                Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
        ]));

        // 画质
        let is_vq_focus = app.download_focus == DownloadFocus::VideoQuality;
        let vq_name = match VIDEO_QUALITIES[app.video_quality_idx] {
                | ybd_core::model::quality::VideoQuality::K8 => "8K 超高清",
                | ybd_core::model::quality::VideoQuality::K4 => "4K 超清",
                | ybd_core::model::quality::VideoQuality::FHD1080P60 => "1080P 60帧 (高码率)",
                | ybd_core::model::quality::VideoQuality::FHD1080P => "1080P 高清",
                | ybd_core::model::quality::VideoQuality::HD720P => "720P 高清",
                | ybd_core::model::quality::VideoQuality::SD480P => "480P 清晰",
                | ybd_core::model::quality::VideoQuality::LD360P => "360P 流畅",
        };
        lines.push(Line::from(vec![
                Span::styled(
                        if is_vq_focus {
                                "  ❯ 画质    "
                        } else {
                                "    画质    "
                        },
                        if is_vq_focus {
                                Style::default().fg(Color::Cyan)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
                Span::styled(
                        format!("{} ▾", vq_name),
                        if is_vq_focus {
                                Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
        ]));

        // 音质
        let is_aq_focus = app.download_focus == DownloadFocus::AudioQuality;
        let aq_name = match AUDIO_QUALITIES[app.audio_quality_idx] {
                | ybd_core::model::quality::AudioQuality::HiRes => "Hi-Res 无损",
                | ybd_core::model::quality::AudioQuality::Dolby => "杜比全景声",
                | ybd_core::model::quality::AudioQuality::High => "192Kbps 高音质",
                | ybd_core::model::quality::AudioQuality::Medium => "132Kbps 标准",
                | ybd_core::model::quality::AudioQuality::Low => "64Kbps",
        };
        lines.push(Line::from(vec![
                Span::styled(
                        if is_aq_focus {
                                "  ❯ 音质    "
                        } else {
                                "    音质    "
                        },
                        if is_aq_focus {
                                Style::default().fg(Color::Cyan)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
                Span::styled(
                        format!("{} ▾", aq_name),
                        if is_aq_focus {
                                Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
        ]));

        // 编码
        let is_ve_focus = app.download_focus == DownloadFocus::VideoEncode;
        let ve_name = match VIDEO_ENCODES[app.video_encode_idx] {
                | ybd_core::model::quality::VideoEncode::AVC => "AVC / H.264 (兼容性最好)",
                | ybd_core::model::quality::VideoEncode::HEVC => "HEVC / H.265 (高压缩比)",
                | ybd_core::model::quality::VideoEncode::AV1 => "AV1 (极高画质)",
        };
        lines.push(Line::from(vec![
                Span::styled(
                        if is_ve_focus {
                                "  ❯ 编码    "
                        } else {
                                "    编码    "
                        },
                        if is_ve_focus {
                                Style::default().fg(Color::Cyan)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
                Span::styled(
                        format!("{} ▾", ve_name),
                        if is_ve_focus {
                                Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
        ]));

        // 批量开关
        let is_batch_focus = app.download_focus == DownloadFocus::BatchToggle;
        let batch_icon = if app.batch { "●" } else { "○" };
        lines.push(Line::from(vec![
                Span::styled(
                        if is_batch_focus {
                                "  ❯ 批量    "
                        } else {
                                "    批量    "
                        },
                        if is_batch_focus {
                                Style::default().fg(Color::Cyan)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
                Span::styled(
                        format!("{} 解析合集 / 收藏夹全量视频", batch_icon),
                        if is_batch_focus {
                                Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
        ]));

        lines.push(Line::from(""));

        // 存储路径
        lines.push(Line::from(vec![Span::styled(
                "  存储路径",
                Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
        )]));

        let is_out_focus = app.download_focus == DownloadFocus::OutputDir;
        let out_display_line_idx = lines.len();
        let path_width = area.width.saturating_sub(14) as usize;
        let (scrolled_out, visual_out_cursor) =
                get_scrolled_text(&app.output_dir_input, app.cursor_pos, path_width);

        lines.push(Line::from(vec![
                Span::styled(
                        if is_out_focus {
                                "  ❯ 输出    "
                        } else {
                                "    输出    "
                        },
                        if is_out_focus {
                                Style::default().fg(Color::Cyan)
                        } else {
                                Style::default().fg(Color::Gray)
                        },
                ),
                Span::styled(
                        if app.output_dir_input.is_empty() && !is_out_focus {
                                "(当前目录)".to_string()
                        } else {
                                scrolled_out
                        },
                        if is_out_focus {
                                Style::default()
                                        .fg(Color::White)
                                        .add_modifier(Modifier::BOLD)
                        } else {
                                Style::default().fg(Color::DarkGray)
                        },
                ),
        ]));

        let is_ff_focus = app.download_focus == DownloadFocus::FfmpegPath;
        let ff_display_line_idx = lines.len();
        let (scrolled_ff, visual_ff_cursor) =
                get_scrolled_text(&app.ffmpeg_path_input, app.cursor_pos, path_width);

        let ff_display = if app.ffmpeg_path_input.is_empty() && !is_ff_focus {
                "(从系统 PATH 查找)".to_string()
        } else {
                scrolled_ff
        };
        lines.push(Line::from(vec![
                Span::styled(
                        if is_ff_focus {
                                "  ❯ FFmpeg  "
                        } else {
                                "    FFmpeg  "
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
                Span::styled("Enter ", Style::default().fg(Color::Cyan)),
                Span::styled("开始下载     ", Style::default().fg(Color::Gray)),
                Span::styled("p ", Style::default().fg(Color::Cyan)),
                Span::styled("解析信息     ", Style::default().fg(Color::Gray)),
                Span::styled("s ", Style::default().fg(Color::Cyan)),
                Span::styled("切换批量     ", Style::default().fg(Color::Gray)),
                Span::styled("c ", Style::default().fg(Color::Cyan)),
                Span::styled("清空链接", Style::default().fg(Color::Gray)),
        ]));

        f.render_widget(Paragraph::new(lines), area);

        // 设置终端真实光标位置（结合横向滚动的视觉偏移）
        if app.mode == VimMode::Insert {
                match app.download_focus {
                        | DownloadFocus::UrlInput => {
                                let x = area.x + 4 + visual_url_cursor;
                                let y = area.y + 2;
                                f.set_cursor_position((x, y));
                        },
                        | DownloadFocus::OutputDir => {
                                let x = area.x + 12 + visual_out_cursor;
                                let y = area.y + out_display_line_idx as u16;
                                f.set_cursor_position((x, y));
                        },
                        | DownloadFocus::FfmpegPath => {
                                let x = area.x + 12 + visual_ff_cursor;
                                let y = area.y + ff_display_line_idx as u16;
                                f.set_cursor_position((x, y));
                        },
                        | _ => {},
                }
        }
}
