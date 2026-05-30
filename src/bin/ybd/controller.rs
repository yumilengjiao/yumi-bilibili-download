use std::{env, sync::Arc};

use futures::future;
use reqwest::Client;
use tokio::sync::Semaphore;
use yumi_bilibili_download::{
    actuator::{self, Mode, get_basic_collection_info},
    client::BiliClient,
    error::{Error, Result},
    model::{download::DownloadOption, quality::VideoEncode, video::PlayUrlResponse},
    progress::{DownloadMutiProgess, DownloadProgress},
    url::UA,
    util::{extract_bv_id, extract_media_id},
};

use crate::{app::App, clap_app::DownloadArgs};

pub async fn start_task(app: &App, args: DownloadArgs) -> Result<()> {
    match args.mode {
        Mode::Audio => download_audio(app, args).await,
        Mode::Cover => download_cover(app, args).await,
        Mode::Video => download_video(app, args).await,
    }
}

async fn download_video(app: &App, args: DownloadArgs) -> Result<()> {
    let DownloadArgs {
        batch,
        output,
        ffmpeg,
        quality_audio,
        quality_video,
        encode_video,
        url,
        ..
    } = args;
    let dir = match output {
        Some(p) => p,
        None => env::current_dir()?,
    };
    let output = dir;
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
        let ml_id = extract_media_id(&url)?;
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
        let semaphore = Arc::new(Semaphore::new((concurrencies / 2).max(1)));
        let mp = Arc::new(std::sync::Mutex::new(DownloadMutiProgess::new(
            vec![],
            bv_ids.len() as u64,
        )));

        for bv_id in bv_ids {
            let bc = Arc::clone(&bc);
            let base_path = Arc::clone(&base_path);
            let sp = Arc::clone(&semaphore);
            let ffmpeg_s = Arc::clone(&ffmpeg);
            let mp = Arc::clone(&mp);
            let jh = tokio::spawn(async move {
                let _permit = sp.acquire().await?;
                let (title, _, _) = actuator::get_basic_video_info(&bv_id, Some(&bc))
                    .await
                    .map_err(|e| {
                        Error::Normal(format!("无法获取视频,BV: {}, 错误信息: {}", bv_id, e))
                    })?;
                let video_path =
                    base_path.join(format!("{}.mp4", sanitize_filename::sanitize(&title)));

                if video_path.exists() {
                    mp.lock().unwrap().inc_total();
                    return Ok(());
                }

                let pur = PlayUrlResponse::new(&bc, &bv_id).await.map_err(|e| {
                    Error::Normal(format!(
                        "无法获取视频: {},BV: {},错误信息: {}",
                        title, bv_id, e
                    ))
                })?;
                let video_tmp = video_path.with_extension("video.tmp");
                let audio_tmp = video_path.with_extension("audio.tmp");

                // 创建进度条，clone pb 给 callback
                let dp_video = DownloadProgress::new(format!("{} [视频]", bv_id), 0);
                let dp_audio = DownloadProgress::new(format!("{} [音频]", bv_id), 0);
                let pb_video = dp_video.pb.clone();
                let pb_audio = dp_audio.pb.clone();
                mp.lock().unwrap().add(dp_video);
                mp.lock().unwrap().add(dp_audio);

                let mut builder = DownloadOption::builder()
                    .video_encode(VideoEncode::AVC)
                    .video_path(&video_tmp)
                    .audio_path(&audio_tmp)
                    .output(&video_path)
                    .on_video_progress(Arc::new(move |downloaded, total| {
                        if let Some(t) = total {
                            pb_video.set_length(t);
                            pb_video.set_position(downloaded);
                            if downloaded >= t {
                                pb_video.finish_and_clear();
                            }
                        } else {
                            pb_video.set_position(downloaded);
                        }
                    }))
                    .on_audio_progress(Arc::new(move |downloaded, total| {
                        if let Some(t) = total {
                            pb_audio.set_length(t);
                            pb_audio.set_position(downloaded);
                            if downloaded >= t {
                                pb_audio.finish_and_clear();
                            }
                        } else {
                            pb_audio.set_position(downloaded);
                        }
                    }));

                if let Some(quality_audio) = quality_audio {
                    builder = builder.audio_quality(quality_audio);
                }

                if let Some(quality_video) = quality_video {
                    builder = builder.video_quality(quality_video);
                }

                if let Some(encode_video) = encode_video {
                    builder = builder.video_encode(encode_video);
                }

                if let Some(ffmpeg) = ffmpeg_s.as_deref() {
                    builder = builder.ffmpeg_path(ffmpeg);
                }

                let option = builder.build();
                let result = actuator::download_video(&bc, &pur, &option)
                    .await
                    .map_err(|e| {
                        Error::Normal(format!(
                            "无法获取视频: {},BV: {},错误信息: {}",
                            title, bv_id, e
                        ))
                    });
                mp.lock().unwrap().inc_total();
                result
            });
            handlers.push(jh);
        }

        let results = future::join_all(handlers).await;

        mp.lock().unwrap().finish();

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
        let bv_id = extract_bv_id(&url)?;
        let (title, _, _) = actuator::get_basic_video_info(&bv_id, Some(&bili_client))
            .await
            .map_err(|e| Error::Normal(format!("无法获取视频,BV: {}, 错误信息: {}", bv_id, e)))?;
        let video_path = output.join(format!("{}.mp4", title));
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

        let dp_video = DownloadProgress::new(format!("{} [视频]", bv_id), 0);
        let dp_audio = DownloadProgress::new(format!("{} [音频]", bv_id), 0);
        let pb_video = dp_video.pb.clone();
        let pb_audio = dp_audio.pb.clone();

        let builder = DownloadOption::builder()
            .video_encode(VideoEncode::AVC)
            .video_path(&video_tmp)
            .audio_path(&audio_tmp)
            .output(&video_path)
            .on_video_progress(Arc::new(move |downloaded, total| {
                if let Some(t) = total {
                    pb_video.set_length(t);
                }
                pb_video.set_position(downloaded);
            }))
            .on_audio_progress(Arc::new(move |downloaded, total| {
                if let Some(t) = total {
                    pb_audio.set_length(t);
                }
                pb_audio.set_position(downloaded);
            }));

        // ...其他 builder 字段

        let option = builder.build();
        actuator::download_video(&bili_client, &pur, &option)
            .await
            .map_err(|e| {
                Error::Normal(format!(
                    "无法获取视频: {},BV: {},错误信息: {}",
                    title, bv_id, e
                ))
            })?;

        dp_video.pb.finish(); // 直接用 dp 的 pb
        dp_audio.pb.finish(); // 直接用 dp 的 pb
    }
    Ok(())
}

async fn download_audio(app: &App, args: DownloadArgs) -> Result<()> {
    let DownloadArgs {
        batch,
        output,
        quality_audio,
        url,
        ..
    } = args;
    let output = match output {
        Some(p) => p,
        None => env::current_dir()?,
    };
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
        let ml_id = extract_media_id(&url)?;
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
        let mp = Arc::new(std::sync::Mutex::new(DownloadMutiProgess::new(
            vec![],
            bv_ids.len() as u64,
        )));

        for bv_id in bv_ids {
            let bc = Arc::clone(&bc);
            let base_path = Arc::clone(&base_path);
            let sp = Arc::clone(&semaphore);
            let mp = Arc::clone(&mp);
            let jh = tokio::spawn(async move {
                let _permit = sp.acquire().await?;
                let (title, _, _) = actuator::get_basic_video_info(&bv_id, Some(&bc))
                    .await
                    .map_err(|e| {
                        Error::Normal(format!("无法获取视频,BV: {}, 错误信息: {}", bv_id, e))
                    })?;
                let audio_path =
                    base_path.join(format!("{}.m4a", sanitize_filename::sanitize(&title)));

                if audio_path.exists() {
                    mp.lock().unwrap().inc_total();
                    return Ok(());
                }

                let pur = PlayUrlResponse::new(&bc, &bv_id).await.map_err(|e| {
                    Error::Normal(format!(
                        "无法获取音频: {},BV: {},错误信息: {}",
                        title, bv_id, e
                    ))
                })?;

                let dp = DownloadProgress::new(bv_id.clone(), 0);
                let pb = dp.pb.clone();
                mp.lock().unwrap().add(dp);

                let mut builder = DownloadOption::builder()
                    .audio_path(&audio_path)
                    .on_audio_progress(Arc::new(move |downloaded, total| {
                        if let Some(t) = total {
                            pb.set_length(t);
                            pb.set_position(downloaded);
                            if downloaded >= t {
                                pb.finish_and_clear();
                            }
                        } else {
                            pb.set_position(downloaded);
                        }
                    }));

                if let Some(quality_audio) = quality_audio {
                    builder = builder.audio_quality(quality_audio);
                }

                let option = builder.build();

                let result = actuator::download_audio(&bc, &pur, &option)
                    .await
                    .map_err(|e| {
                        Error::Normal(format!(
                            "无法获取音频: {},BV: {},错误信息: {}",
                            title, bv_id, e
                        ))
                    });

                mp.lock().unwrap().inc_total();
                result
            });
            handlers.push(jh);
        }

        let results = future::join_all(handlers).await;
        mp.lock().unwrap().finish();

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
        let bv_id = extract_bv_id(&url)?;
        let (title, _, _) = actuator::get_basic_video_info(&bv_id, Some(&bili_client))
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

        let dp = DownloadProgress::new(bv_id.clone(), 0);
        let pb = dp.pb.clone();

        let mut builder = DownloadOption::builder()
            .audio_path(&audio_path)
            .on_audio_progress(Arc::new(move |downloaded, total| {
                if let Some(t) = total {
                    pb.set_length(t);
                }
                pb.set_position(downloaded);
            }));

        if let Some(quality_audio) = quality_audio {
            builder = builder.audio_quality(quality_audio);
        }

        let option = builder.build();

        actuator::download_audio(&bili_client, &pur, &option)
            .await
            .map_err(|e| {
                Error::Normal(format!(
                    "无法获取音频: {},BV: {},错误信息: {}",
                    title, bv_id, e
                ))
            })?;

        dp.pb.finish_and_clear();
    }
    Ok(())
}

async fn download_cover(app: &App, args: DownloadArgs) -> Result<()> {
    let DownloadArgs {
        batch, output, url, ..
    } = args;
    let output = match output {
        Some(p) => p,
        None => env::current_dir()?,
    };
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
    tokio::fs::create_dir_all(&output).await?;
    if batch {
        let ml_id = extract_media_id(&url)?;
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

                // 检测图片是否已经下载
                if cover_path.exists() {
                    return Ok(());
                }

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
        let bv_id = extract_bv_id(&url)?;
        let client = Client::builder().user_agent(UA).build()?;
        let (title, cover, _) = actuator::get_basic_video_info(&bv_id, Some(&bili_client)).await?;
        let cover_path = output.join(format!("{}.png", title));
        actuator::download_cover(&client, &cover, &cover_path).await?;
    }
    Ok(())
}
