use clap::ValueEnum;
use regex::Regex;
use reqwest::Client;
use serde_json::Value;

use crate::{
    error::{Error, Result},
    url::{VEDIO_INFO, WBI},
};

#[derive(Debug, Clone, ValueEnum)]
pub enum Mode {
    Cover,
    Audio,
    Vedio,
}

/// 获取视频基本信息
///
/// * `client`: reqwest请求客户端
/// * `bvid`: 视频bv号
/// * `sessdata`: 用户凭证
///
/// # Retures
///
/// (title, cid)
pub async fn get_basic_video_info(
    client: &Client,
    bvid: &str,
    sessdata: &str,
) -> Result<(String, String)> {
    let resp: Value = client
        .get(VEDIO_INFO)
        .query(&[("bvid", bvid)])
        .header("Cookie", format!("SESSDATA={}", sessdata))
        .send()
        .await?
        .json()
        .await?;

    if resp["code"].as_i64().unwrap_or(-1) != 0 {
        return Err(Error::Normal(format!(
            "获取视频信息失败: {}",
            resp["message"]
        )));
    }

    let title = resp["data"]["title"]
        .as_str()
        .unwrap_or("未知标题")
        .to_string();
    let cid = resp["data"]["cid"]
        .as_i64()
        .ok_or_else(|| Error::Normal("无法获取 cid".into()))?
        .to_string();

    Ok((title, cid))
}

/// 提取bvid
///
/// * `bv_id`: 视频url
fn extract_bv_id(input: &str) -> Result<String> {
    let regex = Regex::new(r"BV[a-z0-9A-Z]+").expect("不正确的正则表达式");
    regex
        .find(input)
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| Error::Normal("无法从输入中提取bv号".into()))
}

/// 获取wbi签名所需的img_key和sub_key密钥
///
/// * `client`: reqwest客户端
/// * `sessdata`: 会话令牌
///
/// # Retures
///
/// (img_key, sub_key)
pub async fn get_wbi_keys(client: &Client, sessdata: &str) -> Result<(String, String)> {
    let resp: Value = client
        .get(WBI)
        .header("Cookie", format!("SESSDATA={}", sessdata))
        .send()
        .await?
        .json()
        .await?;

    let img_url = resp["data"]["wbi_img"]["img_url"]
        .as_str()
        .ok_or(Error::Normal("无法获取 img_url".into()))?;
    let sub_url = resp["data"]["wbi_img"]["sub_url"]
        .as_str()
        .ok_or(Error::Normal("无法获取 sub_url".into()))?;

    // 从 URL 中提取文件名（去掉路径和 .png 后缀）
    let img_key = img_url
        .split('/')
        .next_back()
        .unwrap_or("")
        .trim_end_matches(".png")
        .to_string();
    let sub_key = sub_url
        .split('/')
        .next_back()
        .unwrap_or("")
        .trim_end_matches(".png")
        .to_string();

    Ok((img_key, sub_key))
}
