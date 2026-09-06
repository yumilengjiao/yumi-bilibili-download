//! Bilibili 资源下载终端用户界面工具入口 (ybdtui)
//!
//! 基于 Ratatui + Crossterm 实现纯 Safe Rust、Vim 风格的极简终端 UI。

mod app;
mod cache;
mod config;
mod directories;
mod event;
mod handler;
mod model;
mod service;
mod ui;

use std::{io, time::Duration};

use crossterm::{
        event::{Event, EventStream},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;
use ybd_core::error::Result;

use crate::{app::App, event::AppEvent, handler::handle_key_event, ui::render_ui};

#[tokio::main]
async fn main() -> Result<()> {
        // 终端环境初始化
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // 安装 panic hook 防止终端进入异常状态
        let default_panic = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
                let _ = disable_raw_mode();
                let _ = execute!(io::stdout(), LeaveAlternateScreen);
                default_panic(info);
        }));

        let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();
        let mut app = App::new(tx.clone());

        // 异步输入事件流
        let mut reader = EventStream::new();
        let mut tick_interval = tokio::time::interval(Duration::from_millis(50));

        loop {
                // 绘制界面
                terminal.draw(|f| render_ui(f, &app))?;

                if app.should_quit {
                        break;
                }

                tokio::select! {
                        | Some(app_event) = rx.recv() => {
                                app.handle_event(app_event);
                        }
                        | Some(Ok(crossterm_event)) = reader.next() => {
                                match crossterm_event {
                                        | Event::Key(key) => {
                                                handle_key_event(&mut app, key);
                                        }
                                        | Event::Resize(w, h) => {
                                                app.handle_event(AppEvent::Resize(w, h));
                                        }
                                        | _ => {}
                                }
                        }
                        | _ = tick_interval.tick() => {
                                app.handle_event(AppEvent::Tick);
                        }
                }
        }

        // 恢复终端状态
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
}
