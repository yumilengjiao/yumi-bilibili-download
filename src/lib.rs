//! Bilibili 资源下载与解析核心库
//!
//! 提供视频/音频/封面下载、登录状态轮询、收藏夹抓取以及元数据注入等功能。

pub mod actuator;

pub mod client;
pub mod error;
pub mod login;
pub mod model;
pub mod progress;
pub mod url;
pub mod util;
