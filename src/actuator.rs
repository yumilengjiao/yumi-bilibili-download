use std::path::Path;

use clap::ValueEnum;
use reqwest::Client;
use serde_json::Value;
use tokio::{fs::File, io, process};

use crate::{
    client::BiliClient,
    error::{Error, Result},
    model::{collection::CollectionUrlResponse, download::DownloadOption, video::PlayUrlResponse},
    url::{MEDIO_LIST, UA, VIDEO_INFO},
};

#[derive(Debug, Clone, ValueEnum)]
pub enum Mode {
    Cover,
    Audio,
    Video,
}

/// 下载视频文件
///
/// * `bili_client`: 发送请求的带令牌的客户端
/// * `pur`: 视频信息返回响应体
/// * `video_quality`: [视频分辨率],
/// * `video_encode`: [视频编码格式],
/// * `path`: 下载到的路径
pub async fn download_video(
    bili_client: &BiliClient,
    pur: &PlayUrlResponse,
    download_option: &DownloadOption<'_>,
) -> Result<()> {
    let video_path = download_option
        .video_path
        .ok_or(Error::Path("video_path 未设置".into()))?;
    let audio_path = download_option
        .audio_path
        .ok_or(Error::Path("audio_path 未设置".into()))?;
    let output = download_option
        .output
        .ok_or(Error::Path("output 未设置".into()))?;

    let (video_res, audio_res) = tokio::join!(
        download_video_with_no_audio(bili_client, pur, download_option),
        download_audio(bili_client, pur, download_option),
    );
    video_res?;
    audio_res?;
    merge_video_audio(video_path, audio_path, output, download_option.ffmpeg_path).await?;
    tokio::fs::remove_file(video_path).await?;
    tokio::fs::remove_file(audio_path).await?;
    Ok(())
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
/// * `pur`: 视频信息返回响应体
/// * `video_quality`: [视频分辨率],
/// * `video_encode`: [视频编码格式],
/// * `path`: 下载到的路径
pub async fn download_video_with_no_audio(
    bili_client: &BiliClient,
    pur: &PlayUrlResponse,
    download_option: &DownloadOption<'_>,
) -> Result<()> {
    let video_path = download_option
        .video_path
        .ok_or(Error::Path("没有输入视频输出路径".into()))?;
    if video_path.is_dir() {
        return Err(Error::Path("路径不能是目录".into()));
    }
    let video_data = pur.get_data()?;

    let url = if download_option.video_quality.is_some() || download_option.video_encode.is_some() {
        video_data
            .get_specified_video_url(download_option.video_quality, download_option.video_encode)
            .ok_or(Error::Normal(
                "无法获取指定分辨率或编码格式的视频资源".into(),
            ))?
    } else {
        video_data
            .best_video_quality_url()
            .ok_or(Error::Normal("没有找到视频".into()))?
    };
    download_url(bili_client, url, video_path).await
}

/// 下载音频文件,内部逻辑与视频一致
///
/// * `bili_client`: 发送请求的带令牌的客户端
/// * `pur`: 视频信息返回响应体
/// * `path`: 下载到的路径
pub async fn download_audio(
    bili_client: &BiliClient,
    pur: &PlayUrlResponse,
    download_option: &DownloadOption<'_>,
) -> Result<()> {
    let path = download_option
        .audio_path
        .ok_or(Error::Path("没有输入音频输出路径".into()))?;
    if path.is_dir() {
        return Err(Error::Path("路径不能是目录".into()));
    }
    let video_data = pur.get_data()?;
    let url = match download_option.audio_quality {
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
pub async fn get_basic_video_info(
    bvid: &str,
    bili_client: Option<&BiliClient>,
) -> Result<(String, String, String)> {
    let resp: Value = if let Some(biliclient) = bili_client {
        biliclient
            .get(VIDEO_INFO)
            .header("Referer", "https://www.bilibili.com")
            .query(&[("bvid", bvid)])
            .send()
            .await?
            .json()
            .await?
    } else {
        let client = Client::builder().user_agent(UA).build()?;
        client
            .get(VIDEO_INFO)
            .header("Referer", "https://www.bilibili.com")
            .query(&[("bvid", bvid)])
            .send()
            .await?
            .json()
            .await?
    };

    if resp["code"].as_i64().unwrap_or(-1) != 0 {
        println!("{:#?}", resp["code"]);
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

/// 获取收藏夹信息
///
/// * `ml_id`: 收藏夹标识
/// * `page_number`: 页号
/// * `page_size`: 页内数
/// * `bili_client`: 如果收藏夹是私有未公开需要认证信息
pub async fn get_basic_collection_info(
    ml_id: &str,
    page_number: usize,
    page_size: usize,
    bili_client: Option<&BiliClient>,
) -> Result<CollectionUrlResponse> {
    if let Some(bilibili_client) = bili_client {
        let cur: CollectionUrlResponse = bilibili_client
            .get(MEDIO_LIST)
            .query(&[
                ("media_id", ml_id),
                ("ps", &page_size.to_string()),
                ("pn", &page_number.to_string()),
            ])
            .send()
            .await?
            .json()
            .await?;
        return Ok(cur);
    }
    let client = Client::builder().user_agent(UA).build()?;
    let cur: CollectionUrlResponse = client
        .get(MEDIO_LIST)
        .header("Referer", "https://www.bilibili.com")
        .query(&[
            ("media_id", ml_id),
            ("ps", &page_size.to_string()),
            ("pn", &page_number.to_string()),
        ])
        .send()
        .await?
        .json()
        .await?;
    Ok(cur)
}

pub async fn merge_video_audio(
    video_path: &Path,
    audio_path: &Path,
    output_path: &Path,
    ffmpeg_path: Option<&Path>,
) -> Result<()> {
    let ffmpeg = ffmpeg_path.unwrap_or(Path::new("ffmpeg"));
    let status = process::Command::new(ffmpeg)
        .args([
            "-i",
            video_path.to_str().unwrap(),
            "-i",
            audio_path.to_str().unwrap(),
            "-c",
            "copy",
            output_path.to_str().unwrap(),
        ])
        .status()
        .await?;
    if !status.success() {
        return Err(Error::Normal("ffmpeg 合并失败".into()));
    }
    Ok(())
}

async fn download_url(bili_client: &BiliClient, url: &str, path: &Path) -> Result<()> {
    let mut last_err = None;
    for attempt in 0..3 {
        if attempt > 0 {
            tokio::time::sleep(tokio::time::Duration::from_secs(attempt * 2)).await;
        }
        match try_download_url(bili_client, url, path).await {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.unwrap())
}

async fn try_download_url(bili_client: &BiliClient, url: &str, path: &Path) -> Result<()> {
    let mut res = bili_client.get(url).send().await?;
    let mut file = File::create(path).await?;
    while let Some(chunk) = res.chunk().await? {
        io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
    }
    Ok(())
}
