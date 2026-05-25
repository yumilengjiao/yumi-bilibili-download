use std::path::Path;

use clap::ValueEnum;
use regex::Regex;
use reqwest::Client;
use serde_json::Value;
use tokio::{fs::File, io};

use crate::{
    client::BiliClient,
    error::{Error, Result},
    model::{
        quality::{AudioQuality, VideoEncode, VideoQuality},
        video::VideoData,
    },
    url::{UA, VIDEO_INFO, WBI},
};

#[derive(Debug, Clone, ValueEnum)]
pub enum Mode {
    Cover,
    Audio,
    Video,
}

/// 下载视频封面
///
/// * `client`: reqwest请求客户端
/// * `base_url`: 图片url
/// * `path`: 下载到本地的路径
/// * `sessdata`: 用户凭证
pub async fn download_cover(client: &Client, url: &str, path: &Path) -> Result<()> {
    if path.is_dir() {
        return Err(Error::Path("路径不能是目录".into()));
    }
    let safe_path = path
        .parent()
        .unwrap_or(Path::new("."))
        .join(sanitize_filename::sanitize(
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .as_ref(),
        ));
    let mut res = client.get(url).send().await?;
    let mut file = File::create(safe_path).await?;
    while let Some(chunk) = res.chunk().await? {
        io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
    }
    Ok(())
}

/// 下载视频文件,没有音频
///
/// * `bili_client`: 发送请求的带令牌的客户端
/// * `video_data`: 视频元数据
/// * `video_quality`: [视频分辨率],
/// * `video_encode`: [视频编码格式],
/// * `path`: 下载到的路径
pub async fn download_video_with_no_audio(
    bili_client: &BiliClient,
    video_data: &VideoData,
    video_quality: Option<VideoQuality>,
    video_encode: Option<VideoEncode>,
    path: &Path,
) -> Result<()> {
    if path.is_dir() {
        return Err(Error::Path("路径不能是目录".into()));
    }
    let url = if video_quality.is_some() || video_encode.is_some() {
        video_data
            .get_specified_video_url(video_quality, video_encode)
            .ok_or(Error::Normal(
                "无法获取指定分辨率或编码格式的视频资源".into(),
            ))?
    } else {
        video_data
            .best_video_quality_url()
            .ok_or(Error::Normal("没有找到视频".into()))?
    };
    download_url(bili_client, url, path).await
}

/// 下载音频文件,内部逻辑与视频一致
///
/// * `bili_client`: 发送请求的带令牌的客户端
/// * `video_data`: 视频元数据
/// * `path`: 下载到的路径
pub async fn download_audio(
    bili_client: &BiliClient,
    video_data: &VideoData,
    audio_quality: Option<AudioQuality>,
    path: &Path,
) -> Result<()> {
    if path.is_dir() {
        return Err(Error::Path("路径不能是目录".into()));
    }

    let url = match audio_quality {
        Some(aq) => video_data
            .get_specified_audio_url(aq)
            .ok_or(Error::Normal("无法获取指定音质的音频资源".into()))?,
        None => video_data
            .best_audio_url()
            .ok_or(Error::Normal("没有找到音频".into()))?,
    };
    download_url(bili_client, url, path).await
}

/// 获取视频基本信息,基本信息不需要SESSDATA
///
/// * `client`: reqwest请求客户端
/// * `bvid`: 视频bv号
/// * `sessdata`: 用户凭证
///
/// # Retures
///
/// (title, pic, cid) -> 视频标题，视频封面url, 分p标识号
pub async fn get_basic_video_info(bvid: &str) -> Result<(String, String, String)> {
    let client = Client::builder().user_agent(UA).build()?;

    let resp: Value = client
        .get(VIDEO_INFO)
        .header("Referer", "https://www.bilibili.com")
        .query(&[("bvid", bvid)])
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
    let pic = resp["data"]["pic"]
        .as_str()
        .ok_or_else(|| Error::Normal("无法获取视频封面url".into()))?
        .to_string();

    Ok((title, pic, cid))
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
pub async fn get_wbi_keys(client: &BiliClient) -> Result<(String, String)> {
    let resp: Value = client.get(WBI).send().await?.json().await?;

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

async fn download_url(bili_client: &BiliClient, url: &str, path: &Path) -> Result<()> {
    let mut res = bili_client.get(url).send().await?;
    let mut file = File::create(path).await?;
    while let Some(chunk) = res.chunk().await? {
        io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
    }
    Ok(())
}
