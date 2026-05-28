use std::{env, path::PathBuf, sync::Arc};

use futures::future;
use reqwest::Client;
use tokio::sync::Semaphore;
use yumi_bilibili_download::{
    actuator::{self, Mode, get_basic_collection_info},
    client::BiliClient,
    error::{Error, Result},
    model::{download::DownloadOption, quality::VideoEncode, video::PlayUrlResponse},
    url::UA,
    util::{extract_bv_id, extract_media_id},
};

use crate::{app::App, clap_app::DownloadArgs};

pub async fn start_task(app: &App, args: DownloadArgs) -> Result<()> {
    let DownloadArgs {
        mode,
        batch,
        output,
        ffmpeg,
        url,
    } = args;

    let dir = match output {
        Some(p) => p,
        None => env::current_dir()?,
    };
    match mode {
        Mode::Audio => download_audio(app, batch, &url, dir).await,
        Mode::Cover => download_cover(app, batch, &url, dir).await,
        Mode::Video => download_video(app, ffmpeg, batch, &url, dir).await,
    }
}

async fn download_video(
    app: &App,
    ffmpeg: Option<PathBuf>,
    batch: bool,
    url: &str,
    output: PathBuf,
) -> Result<()> {
    tokio::fs::create_dir_all(&output).await?;
    let account = app
        .account
        .as_ref()
        .ok_or(Error::Normal("未登录，请先登录".into()))
        .and_then(|a| {
            if a.is_expired() {
                Err(Error::Normal("登录已过期，请重新登录".into()))
            } else {
                Ok(a)
            }
        })?;
    let bili_client = BiliClient::new(account)?;
    if batch {
        let ml_id = extract_media_id(url)?;
        let concurrencies = app.config.concurrencies;
        let mut bv_ids = Vec::<String>::new();
        let mut pn = 1usize;
        loop {
            let resp =
                get_basic_collection_info(&ml_id, pn, concurrencies, Some(&bili_client)).await?;
            let data = resp.get_data()?;
            let bvids: Vec<String> = data.medias.iter().map(|v| v.bvid.clone()).collect();
            bv_ids.extend(bvids);
            if !data.has_more {
                break;
            }
            pn += 1;
        }

        let bc = Arc::new(bili_client);
        let base_path = Arc::new(output);
        let ffmpeg = Arc::new(ffmpeg);
        let mut handlers = Vec::new();
        let semaphore = Arc::new(Semaphore::new(concurrencies));

        for bv_id in bv_ids {
            let bc = Arc::clone(&bc);
            let base_path = Arc::clone(&base_path);
            let sp = Arc::clone(&semaphore);
            let ffmpeg_s = Arc::clone(&ffmpeg);
            let jh = tokio::spawn(async move {
                let _permit = sp.acquire().await?;
                let (title, _, _) = actuator::get_basic_video_info(&bv_id).await.map_err(|e| {
                    Error::Normal(format!("无法获取视频,BV: {}, 错误信息: {}", bv_id, e))
                })?;
                let video_path =
                    base_path.join(format!("{}.mp4", sanitize_filename::sanitize(&title)));
                let pur = PlayUrlResponse::new(&bc, &bv_id).await.map_err(|e| {
                    Error::Normal(format!(
                        "无法获取视频: {},BV: {},错误信息: {}",
                        title, bv_id, e
                    ))
                })?;
                let video_tmp = video_path.with_extension("video.tmp");
                let audio_tmp = video_path.with_extension("audio.tmp");

                let mut builder = DownloadOption::builder()
                    .video_encode(VideoEncode::AVC)
                    .video_path(&video_tmp)
                    .audio_path(&audio_tmp)
                    .output(&video_path);

                if let Some(ffmpeg) = ffmpeg_s.as_deref() {
                    builder = builder.ffmpeg_path(ffmpeg);
                }

                let option = builder.build();
                actuator::download_video(&bc, &pur, &option)
                    .await
                    .map_err(|e| {
                        Error::Normal(format!(
                            "无法获取视频: {},BV: {},错误信息: {}",
                            title, bv_id, e
                        ))
                    })
            });
            handlers.push(jh);
        }

        let results = future::join_all(handlers).await;

        let failed: Vec<String> = results
            .into_iter()
            .filter_map(|r| match r {
                Ok(Ok(())) => None,
                Ok(Err(e)) => Some(e.to_string()),
                Err(e) => Some(e.to_string()),
            })
            .collect();

        if !failed.is_empty() {
            return Err(Error::Normal(format!(
                "共有 {} 个视频下载失败:\n{}",
                failed.len(),
                failed
                    .iter()
                    .map(|e| format!("\t- {}", e))
                    .collect::<Vec<_>>()
                    .join("\n")
            )));
        }
    } else {
        let bv_id = extract_bv_id(url)?;
        let (title, _, _) = actuator::get_basic_video_info(&bv_id)
            .await
            .map_err(|e| Error::Normal(format!("无法获取视频,BV: {}, 错误信息: {}", bv_id, e)))?;
        let video_path = output.join(format!("{}.mpa", title));
        let pur = PlayUrlResponse::new(&bili_client, &bv_id)
            .await
            .map_err(|e| {
                Error::Normal(format!(
                    "无法获取视频: {},BV: {},错误信息: {}",
                    title, bv_id, e
                ))
            })?;
        let video_tmp = video_path.with_extension("video.tmp");
        let audio_tmp = video_path.with_extension("audio.tmp");

        let mut builder = DownloadOption::builder()
            .video_encode(VideoEncode::AVC)
            .video_path(&video_tmp)
            .audio_path(&audio_tmp)
            .output(&video_path);

        if let Some(ffmpeg) = ffmpeg.as_deref() {
            builder = builder.ffmpeg_path(ffmpeg);
        }

        let option = builder.build();
        actuator::download_video(&bili_client, &pur, &option)
            .await
            .map_err(|e| {
                Error::Normal(format!(
                    "无法获取视频: {},BV: {},错误信息: {}",
                    title, bv_id, e
                ))
            })?;
    }
    Ok(())
}

async fn download_audio(app: &App, batch: bool, url: &str, output: PathBuf) -> Result<()> {
    tokio::fs::create_dir_all(&output).await?;
    let account = app
        .account
        .as_ref()
        .ok_or(Error::Normal("未登录，请先登录".into()))
        .and_then(|a| {
            if a.is_expired() {
                Err(Error::Normal("登录已过期，请重新登录".into()))
            } else {
                Ok(a)
            }
        })?;
    let bili_client = BiliClient::new(account)?;

    if batch {
        let ml_id = extract_media_id(url)?;
        let concurrencies = app.config.concurrencies;
        let mut bv_ids = Vec::<String>::new();
        let mut pn = 1usize;
        loop {
            let resp =
                get_basic_collection_info(&ml_id, pn, concurrencies, Some(&bili_client)).await?;
            let data = resp.get_data()?;
            let bvids: Vec<String> = data.medias.iter().map(|v| v.bvid.clone()).collect();
            bv_ids.extend(bvids);
            if !data.has_more {
                break;
            }
            pn += 1;
        }

        let bc = Arc::new(bili_client);
        let base_path = Arc::new(output);
        let mut handlers = Vec::new();
        let semaphore = Arc::new(Semaphore::new(concurrencies));

        for bv_id in bv_ids {
            let bc = Arc::clone(&bc);
            let base_path = Arc::clone(&base_path);
            let sp = Arc::clone(&semaphore);
            let jh = tokio::spawn(async move {
                let _permit = sp.acquire().await?;
                let (title, _, _) = actuator::get_basic_video_info(&bv_id).await.map_err(|e| {
                    Error::Normal(format!("无法获取视频,BV: {}, 错误信息: {}", bv_id, e))
                })?;
                let audio_path =
                    base_path.join(format!("{}.m4a", sanitize_filename::sanitize(&title)));
                let pur = PlayUrlResponse::new(&bc, &bv_id).await.map_err(|e| {
                    Error::Normal(format!(
                        "无法获取音频: {},BV: {},错误信息: {}",
                        title, bv_id, e
                    ))
                })?;
                let option = DownloadOption::builder().audio_path(&audio_path).build();
                actuator::download_audio(&bc, &pur, &option)
                    .await
                    .map_err(|e| {
                        Error::Normal(format!(
                            "无法获取音频: {},BV: {},错误信息: {}",
                            title, bv_id, e
                        ))
                    })
            });
            handlers.push(jh);
        }

        let results = future::join_all(handlers).await;

        let failed: Vec<String> = results
            .into_iter()
            .filter_map(|r| match r {
                Ok(Ok(())) => None,
                Ok(Err(e)) => Some(e.to_string()),
                Err(e) => Some(e.to_string()),
            })
            .collect();

        if !failed.is_empty() {
            return Err(Error::Normal(format!(
                "共有 {} 个音频下载失败:\n{}",
                failed.len(),
                failed
                    .iter()
                    .map(|e| format!("\t- {}", e))
                    .collect::<Vec<_>>()
                    .join("\n")
            )));
        }
    } else {
        let bv_id = extract_bv_id(url)?;
        let (title, _, _) = actuator::get_basic_video_info(&bv_id)
            .await
            .map_err(|e| Error::Normal(format!("无法获取音频,BV: {}, 错误信息: {}", bv_id, e)))?;
        let audio_path = output.join(format!("{}.m4a", title));
        let pur = PlayUrlResponse::new(&bili_client, &bv_id)
            .await
            .map_err(|e| {
                Error::Normal(format!(
                    "无法获取音频: {},BV: {},错误信息: {}",
                    title, bv_id, e
                ))
            })?;
        let option = DownloadOption::builder().audio_path(&audio_path).build();
        actuator::download_audio(&bili_client, &pur, &option)
            .await
            .map_err(|e| {
                Error::Normal(format!(
                    "无法获取音频: {},BV: {},错误信息: {}",
                    title, bv_id, e
                ))
            })?;
    }
    Ok(())
}

async fn download_cover(app: &App, batch: bool, url: &str, output: PathBuf) -> Result<()> {
    tokio::fs::create_dir_all(&output).await?;
    if batch {
        let account = app
            .account
            .as_ref()
            .ok_or(Error::Normal(
                "未登录，请先登录, 批量下载需要登录账号".into(),
            ))
            .and_then(|a| {
                if a.is_expired() {
                    Err(Error::Normal(
                        "登录已过期，请重新登录, 批量下载需要登录账号".into(),
                    ))
                } else {
                    Ok(a)
                }
            })?;
        let bili_client = BiliClient::new(account)?;
        let ml_id = extract_media_id(url)?;
        let concurrencies = app.config.concurrencies;
        let mut bv_title_covers = Vec::<(String, String, String)>::new();
        let mut pn = 1usize;
        loop {
            let resp =
                get_basic_collection_info(&ml_id, pn, concurrencies, Some(&bili_client)).await?;
            let data = resp.get_data()?;
            let cover_part: Vec<(String, String, String)> = data
                .medias
                .iter()
                .map(|v| {
                    let title = sanitize_filename::sanitize(&v.title);
                    (v.bv_id.clone(), title, v.cover.clone())
                })
                .collect();
            bv_title_covers.extend(cover_part);
            if !data.has_more {
                break;
            }
            pn += 1;
        }

        let bc = Arc::new(bili_client);
        let base_path = Arc::new(output);
        let mut handlers = Vec::new();
        let semaphore = Arc::new(Semaphore::new(concurrencies));

        for btc in bv_title_covers {
            let bc = Arc::clone(&bc);
            let base_path = Arc::clone(&base_path);
            let sp = Arc::clone(&semaphore);
            let jh = tokio::spawn(async move {
                let _permit = sp.acquire().await?;
                let cover_path =
                    base_path.join(format!("{}.png", sanitize_filename::sanitize(&btc.1)));
                actuator::download_cover(bc.downgrade(), &btc.2, &cover_path)
                    .await
                    .map_err(|e| {
                        Error::Normal(format!(
                            "无法获取指定视频封面,title: {},BV: {}, 错误信息: {}",
                            &btc.1, &btc.0, e
                        ))
                    })
            });
            handlers.push(jh);
        }

        let results = future::join_all(handlers).await;

        let failed: Vec<String> = results
            .into_iter()
            .filter_map(|r| match r {
                Ok(Ok(())) => None,
                Ok(Err(e)) => Some(e.to_string()),
                Err(e) => Some(e.to_string()),
            })
            .collect();

        if !failed.is_empty() {
            return Err(Error::Normal(format!(
                "共有 {} 个封面下载失败:\n{}",
                failed.len(),
                failed
                    .iter()
                    .map(|e| format!("\t- {}", e))
                    .collect::<Vec<_>>()
                    .join("\n")
            )));
        }
    } else {
        let bv_id = extract_bv_id(url)?;
        let client = Client::builder().user_agent(UA).build()?;
        let (title, cover, _) = actuator::get_basic_video_info(&bv_id).await?;
        let cover_path = output.join(format!("{}.png", title));
        actuator::download_cover(&client, &cover, &cover_path).await?;
    }
    Ok(())
}
