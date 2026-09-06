//! 下载任务列表页面渲染模块（支持纵向智能跟随滚动与屏幕视觉焦点对齐）

use ratatui::{
        Frame,
        layout::Rect,
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::Paragraph,
};

use crate::{
        app::App,
        model::{DownloadMode, TaskStatus},
};

pub fn render_tasks_view(
        f: &mut Frame,
        app: &App,
        area: Rect,
) {
        let mut lines: Vec<Line> = Vec::new();
        let mut task_line_indices: Vec<usize> = Vec::new();

        lines.push(Line::from(""));

        let active_tasks: Vec<_> = app
                .tasks
                .iter()
                .filter(|t| t.status != TaskStatus::Completed)
                .collect();
        let completed_tasks: Vec<_> = app
                .tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Completed)
                .collect();

        let mut visual_idx = 0usize;

        lines.push(Line::from(vec![Span::styled(
                format!("  进行中任务 ({})", active_tasks.len()),
                Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
        )]));

        if active_tasks.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                        "    暂无进行中的下载任务 (前往 [资源下载] 页面提交链接)",
                        Style::default().fg(Color::DarkGray),
                )]));
                lines.push(Line::from(""));
        } else {
                lines.push(Line::from(""));
                for task in &active_tasks {
                        let is_selected = app.selected_task_idx == visual_idx;
                        let prefix = if is_selected { "  ❯ " } else { "    " };

                        let mode_tag = match task.mode {
                                | DownloadMode::Video => "[视频]",
                                | DownloadMode::Audio => "[音频]",
                                | DownloadMode::Cover => "[封面]",
                        };

                        task_line_indices.push(lines.len());
                        visual_idx += 1;

                        lines.push(Line::from(vec![
                                Span::styled(
                                        prefix,
                                        if is_selected {
                                                Style::default().fg(Color::Cyan)
                                        } else {
                                                Style::default().fg(Color::DarkGray)
                                        },
                                ),
                                Span::styled(
                                        format!("{} ", mode_tag),
                                        Style::default().fg(Color::LightCyan),
                                ),
                                Span::styled(
                                        &task.title,
                                        if is_selected {
                                                Style::default()
                                                        .fg(Color::White)
                                                        .add_modifier(Modifier::BOLD)
                                        } else {
                                                Style::default().fg(Color::Gray)
                                        },
                                ),
                        ]));

                        match &task.status {
                                | TaskStatus::Downloading => {
                                        if task.mode == DownloadMode::Video {
                                                let (pct_v, bar_v) = make_progress_bar(
                                                        task.video_downloaded,
                                                        task.video_total,
                                                );
                                                let (pct_a, bar_a) = make_progress_bar(
                                                        task.audio_downloaded,
                                                        task.audio_total,
                                                );

                                                lines.push(Line::from(vec![
                                                        Span::styled(
                                                                "      视频  ",
                                                                Style::default()
                                                                        .fg(Color::DarkGray),
                                                        ),
                                                        Span::styled(
                                                                bar_v,
                                                                Style::default().fg(Color::Cyan),
                                                        ),
                                                        Span::styled(
                                                                format!("  {:.1}%", pct_v),
                                                                Style::default().fg(Color::White),
                                                        ),
                                                        Span::styled(
                                                                format!(
                                                                        "  ({} / {})",
                                                                        format_bytes(
                                                                                task.video_downloaded,
                                                                        ),
                                                                        task.video_total
                                                                                .map(format_bytes)
                                                                                .unwrap_or_else(
                                                                                        || {
                                                                                                "未知".into()
                                                                                        },
                                                                                )
                                                                ),
                                                                Style::default()
                                                                        .fg(Color::DarkGray),
                                                        ),
                                                ]));

                                                lines.push(Line::from(vec![
                                                        Span::styled(
                                                                "      音频  ",
                                                                Style::default()
                                                                        .fg(Color::DarkGray),
                                                        ),
                                                        Span::styled(
                                                                bar_a,
                                                                Style::default().fg(Color::Green),
                                                        ),
                                                        Span::styled(
                                                                format!("  {:.1}%", pct_a),
                                                                Style::default().fg(Color::White),
                                                        ),
                                                        Span::styled(
                                                                format!(
                                                                        "  ({} / {})",
                                                                        format_bytes(
                                                                                task.audio_downloaded,
                                                                        ),
                                                                        task.audio_total
                                                                                .map(format_bytes)
                                                                                .unwrap_or_else(
                                                                                        || {
                                                                                                "未知".into()
                                                                                        },
                                                                                )
                                                                ),
                                                                Style::default()
                                                                        .fg(Color::DarkGray),
                                                        ),
                                                ]));
                                        } else {
                                                let (pct_a, bar_a) = make_progress_bar(
                                                        task.audio_downloaded,
                                                        task.audio_total,
                                                );
                                                lines.push(Line::from(vec![
                                                        Span::styled(
                                                                "      进度  ",
                                                                Style::default()
                                                                        .fg(Color::DarkGray),
                                                        ),
                                                        Span::styled(
                                                                bar_a,
                                                                Style::default().fg(Color::Cyan),
                                                        ),
                                                        Span::styled(
                                                                format!("  {:.1}%", pct_a),
                                                                Style::default().fg(Color::White),
                                                        ),
                                                ]));
                                        }
                                },
                                | TaskStatus::Merging => {
                                        lines.push(Line::from(vec![
                                                Span::styled(
                                                        "      状态: ",
                                                        Style::default().fg(Color::DarkGray),
                                                ),
                                                Span::styled(
                                                        "正在合并音视频轨 (FFmpeg)...",
                                                        Style::default().fg(Color::Yellow),
                                                ),
                                        ]));
                                },
                                | TaskStatus::Failed(e) => {
                                        lines.push(Line::from(vec![
                                                Span::styled(
                                                        "      错误: ",
                                                        Style::default().fg(Color::Red),
                                                ),
                                                Span::styled(
                                                        e,
                                                        Style::default().fg(Color::LightRed),
                                                ),
                                        ]));
                                },
                                | _ => {},
                        }
                        lines.push(Line::from(""));
                }
        }

        // 已完成列表
        let completed_count = completed_tasks.len();
        let total_task_count = app.tasks.len();

        lines.push(Line::from(vec![Span::styled(
                format!("  已完成记录 ({})", completed_count),
                Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
        )]));

        if completed_tasks.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                        "    暂无历史下载记录",
                        Style::default().fg(Color::DarkGray),
                )]));
        } else {
                lines.push(Line::from(""));
                for (completed_idx, task) in completed_tasks.into_iter().enumerate() {
                        let progress_tag = if total_task_count > 1 {
                                format!("[{}/{}] ", completed_idx + 1, total_task_count)
                        } else {
                                format!("[{}] ", completed_idx + 1)
                        };

                        lines.push(Line::from(vec![
                                Span::styled("    ✓ ", Style::default().fg(Color::Green)),
                                Span::styled(progress_tag, Style::default().fg(Color::LightGreen)),
                                Span::styled(&task.title, Style::default().fg(Color::Gray)),
                                Span::styled("  [已完成]", Style::default().fg(Color::DarkGray)),
                        ]));
                }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
                Span::styled("  快捷操作: ", Style::default().fg(Color::DarkGray)),
                Span::styled("j / k ", Style::default().fg(Color::Cyan)),
                Span::styled("上下移动选择     ", Style::default().fg(Color::Gray)),
                Span::styled("d ", Style::default().fg(Color::Cyan)),
                Span::styled("移除任务     ", Style::default().fg(Color::Gray)),
                Span::styled("c ", Style::default().fg(Color::Cyan)),
                Span::styled("清空已完成", Style::default().fg(Color::Gray)),
        ]));

        // 动态计算可视滚动窗口（保持当前选中项在视图内）
        let view_height = area.height as usize;
        let total_lines = lines.len();

        let target_line = if app.selected_task_idx < task_line_indices.len() {
                task_line_indices[app.selected_task_idx]
        } else {
                0
        };

        let scroll_offset = if total_lines <= view_height {
                0
        } else if target_line + 4 >= view_height {
                (target_line + 4)
                        .saturating_sub(view_height)
                        .min(total_lines.saturating_sub(view_height))
        } else {
                0
        };

        let visible_lines: Vec<Line> = lines
                .into_iter()
                .skip(scroll_offset)
                .take(view_height)
                .collect();

        f.render_widget(Paragraph::new(visible_lines), area);
}

fn make_progress_bar(
        downloaded: u64,
        total: Option<u64>,
) -> (f64, String) {
        let total_bytes = total.unwrap_or(0);
        let ratio = if total_bytes > 0 {
                (downloaded as f64 / total_bytes as f64).clamp(0.0, 1.0)
        } else {
                0.0
        };

        let width: usize = 30;
        let filled = (ratio * width as f64) as usize;
        let empty = width.saturating_sub(filled);

        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        (ratio * 100.0, bar)
}

fn format_bytes(bytes: u64) -> String {
        if bytes >= 1024 * 1024 * 1024 {
                format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
                format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
                format!("{:.0} KB", bytes as f64 / 1024.0)
        } else {
                format!("{} B", bytes)
        }
}
