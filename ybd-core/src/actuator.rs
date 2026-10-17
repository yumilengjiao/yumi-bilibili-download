//! 核心执行器模块
//!
//! 包含视频、音频、封面下载及使用 ffmpeg 进行音视频合并的高层 API。

use std::path::{Path, PathBuf};

use clap::ValueEnum;
use futures::StreamExt;
use reqwest::Client;
use serde_json::Value;
use tokio::{fs::File, io, process};

use crate::{
        client::BiliClient,
        error::{Error, Result},
        model::{
                collection::CollectionUrlResponse,
                detail::{VideoDetail, VideoInfoResponse},
                download::DownloadOption,
                video::PlayUrlResponse,
        },
        url::{MEDIO_LIST, UA, VIDEO_INFO, VIDEO_TAGS},
        util,
};

/// 一站式写入元数据 + 封面
///
/// 已写入且内容一致的文件会被跳过，不重复写盘。
/// 标签获取、封面下载失败都不会中断主流程。
///
/// * `client`: 带凭证的客户端
/// * `file_path`: 目标 m4a/mp4 文件
/// * `detail`: 视频详细信息（来自 [`get_video_detail`]）
/// * `album`: 合集/收藏夹名，`None` 时使用分区名
/// * `cid`: 当前下载的分P，`None` 时不做分P 标记
pub async fn embed_metadata(
        client: &BiliClient,
        file_path: &Path,
        detail: &VideoDetail,
        album: Option<&str>,
        cid: Option<i64>,
) -> Result<()> {
        let mut meta = detail.media_meta(album, cid);

        // 先做轻量判断，避免每次运行都重新拉标签/封面
        let meta_ok = util::check_metadata(file_path, &meta).unwrap_or(false);
        let cover_ok = util::check_cover_box(file_path).unwrap_or(false);
        if meta_ok && cover_ok {
                return Ok(());
        }

        if !meta_ok {
                meta.lyrics = util::read_lyrics_sidecar(file_path);
                meta.keywords = get_video_tags(&detail.bvid, Some(client)).await;
        }

        let cover = if cover_ok || detail.pic.is_empty() {
                None
        } else {
                util::download_cover_bytes(client, &detail.pic).await.ok()
        };

        match util::write_metadata_with_cover(file_path, &meta, cover) {
                Ok(()) => Ok(()),
                // 封面格式不支持（比如 webp）时退化成只写元数据
                Err(e) => {
                        util::write_metadata(file_path, &meta).map_err(|_| e)?;
                        Ok(())
                },
        }
}

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
pub async fn download_cover(
        client: &Client,
        url: &str,
        path: &Path,
) -> Result<()> {
        if path.is_dir() {
                return Err(Error::Path("路径不能是目录".into()));
        }
        let mut res = client.get(url).send().await?;
        let mut file = File::create(sanitize_path(path)).await?;
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

        let url = if download_option.video_quality.is_some()
                || download_option.video_encode.is_some()
        {
                video_data
                        .get_specified_video_url(
                                download_option.video_quality,
                                download_option.video_encode,
                        )
                        .ok_or(Error::Normal(
                                "无法获取指定分辨率或编码格式的视频资源".into(),
                        ))?
        } else {
                video_data
                        .best_video_quality_url()
                        .ok_or(Error::Normal("没有找到视频".into()))?
        };

        download_url(
                bili_client,
                url,
                video_path,
                download_option.on_video_progress.as_deref(),
        )
        .await
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

        download_url(
                bili_client,
                url,
                path,
                download_option.on_audio_progress.as_deref(),
        )
        .await
}

/// 获取视频完整信息（标题、UP 主、简介、发布时间、分P 列表等）
///
/// 基本信息不需要 SESSDATA，但部分受限视频需要。
///
/// * `bvid`: 视频bv号
/// * `bili_client`: 带凭证的客户端，`None` 时使用匿名客户端
pub async fn get_video_detail(
        bvid: &str,
        bili_client: Option<&BiliClient>,
) -> Result<VideoDetail> {
        let query: [(&str, &str); 1] = [("bvid", bvid)];
        let resp: VideoInfoResponse = match bili_client {
                Some(client) => {
                        client.get(VIDEO_INFO)
                                .header("Referer", "https://www.bilibili.com")
                                .query(&query)
                                .send()
                                .await?
                                .json()
                                .await?
                },
                None => {
                        let client = Client::builder().user_agent(UA).build()?;
                        client.get(VIDEO_INFO)
                                .header("Referer", "https://www.bilibili.com")
                                .query(&query)
                                .send()
                                .await?
                                .json()
                                .await?
                },
        };
        resp.into_detail()
}

/// 获取视频标签（作为 `keywords` 写入元数据）
///
/// 失败时返回空列表，不阻塞下载流程。
pub async fn get_video_tags(
        bvid: &str,
        bili_client: Option<&BiliClient>,
) -> Vec<String> {
        let resp: Value = match bili_client {
                Some(client) => {
                        match client
                                .get(VIDEO_TAGS)
                                .header("Referer", "https://www.bilibili.com")
                                .query(&[("bvid", bvid)])
                                .send()
                                .await
                        {
                                Ok(r) => r.json().await.unwrap_or_default(),
                                Err(_) => Value::Null,
                        }
                },
                None => Value::Null,
        };

        resp["data"]
                .as_array()
                .map(|tags| {
                        tags.iter()
                                .filter_map(|tag| tag["tag_name"].as_str())
                                .map(|name| name.to_string())
                                .collect()
                })
                .unwrap_or_default()
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
                client.get(VIDEO_INFO)
                        .header("Referer", "https://www.bilibili.com")
                        .query(&[("bvid", bvid)])
                        .send()
                        .await?
                        .json()
                        .await?
        };

        let detail = serde_json::from_value::<VideoInfoResponse>(resp)?.into_detail()?;

        let pic = if detail.pic.is_empty() {
                return Err(Error::Normal("无法获取视频封面url".into()));
        } else {
                detail.pic.clone()
        };
        if detail.cid == 0 {
                return Err(Error::Normal("无法获取 cid".into()));
        }
        let title = if detail.title.is_empty() {
                "未知标题".to_string()
        } else {
                detail.title.clone()
        };

        Ok((title, pic, detail.cid.to_string()))
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
        let video_str = video_path
                .to_str()
                .ok_or(Error::Path("视频临时路径包含非法编码字符".into()))?;
        let audio_str = audio_path
                .to_str()
                .ok_or(Error::Path("音频临时路径包含非法编码字符".into()))?;
        let output_str = output_path
                .to_str()
                .ok_or(Error::Path("输出路径包含非法编码字符".into()))?;

        let output = process::Command::new(ffmpeg)
                .args([
                        "-y", "-i", video_str, "-i", audio_str, "-c", "copy", output_str,
                ])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .output()
                .await?;
        if !output.status.success() {
                let err_msg = String::from_utf8_lossy(&output.stderr);
                let clean_msg = err_msg.lines().next_back().unwrap_or("FFmpeg 合并失败");
                return Err(Error::Normal(format!("FFmpeg 合并失败: {}", clean_msg)));
        }
        Ok(())
}

async fn download_url(
        bili_client: &BiliClient,
        url: &str,
        path: &Path,
        on_progress: Option<&(dyn Fn(u64, Option<u64>) + Send + Sync)>,
) -> Result<()> {
        if path.is_dir() {
                return Err(Error::Path("路径不能是目录".into()));
        }

        let safe_path = sanitize_path(path);

        let mut last_err = None;
        for attempt in 0..3 {
                if attempt > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_secs(attempt * 2)).await;
                }
                match try_download_url(bili_client, url, &safe_path, on_progress).await {
                        Ok(()) => return Ok(()),
                        Err(e) => last_err = Some(e),
                }
        }
        Err(last_err.unwrap_or(Error::Normal("下载失败，无可用重试结果".into())))
}

async fn try_download_url(
        bili_client: &BiliClient,
        url: &str,
        path: &Path,
        on_progress: Option<&(dyn Fn(u64, Option<u64>) + Send + Sync)>,
) -> Result<()> {
        let res = bili_client.get(url).send().await?;
        let total = res.content_length(); // 这里拿总大小
        let mut file = File::create(path).await?;
        let mut downloaded = 0u64;
        let mut stream = res.bytes_stream();
        while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                downloaded += chunk.len() as u64;
                io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
                if let Some(cb) = on_progress {
                        cb(downloaded, total);
                }
        }
        Ok(())
}

/// 清理路径中的文件名，替换掉 Windows/Linux 下非法的字符（如 `/`、`\`、`:`、`"` 等），
/// 只保留目录部分不变，避免视频标题中含特殊字符时创建文件失败
///
/// * `path`: 原始路径
fn sanitize_path(path: &Path) -> PathBuf {
        path.parent()
                .unwrap_or(Path::new("."))
                .join(sanitize_filename::sanitize(
                        path.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .as_ref(),
                ))
}
