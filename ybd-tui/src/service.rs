//! 核心业务异步服务模块
//!
//! 负责与 ybd-core 交互执行账号查询、扫码轮询、视频解析与下载流转。

use std::{path::PathBuf, sync::Arc, time::SystemTime};

use futures::future;
use qrcode::{QrCode, render::unicode};
use reqwest::Client;
use serde_json::Value;
use tokio::sync::{Semaphore, mpsc::UnboundedSender};
use ybd_core::{
        actuator::{self, get_basic_collection_info},
        client::BiliClient,
        model::{
                account::Account, download::DownloadOption, quality::VideoEncode,
                video::PlayUrlResponse,
        },
        url::{LOGIN, UA, VALIDATE_QRCODE, VIDEO_INFO, WBI},
        util,
};

use crate::{
        event::AppEvent,
        model::{DownloadMode, QrLoginStatus, UserProfile, VideoInfoPreview},
};

pub async fn fetch_user_profile(
        account: &Account,
        tx: UnboundedSender<AppEvent>,
) {
        let uid = account.get_user_id().to_string();
        let sessdata = account.get_sessdata();
        let sessdata_preview = if sessdata.len() > 10 {
                format!("{}...{}", &sessdata[0..4], &sessdata[sessdata.len() - 4..])
        } else {
                "******".to_string()
        };

        if let Ok(client) = BiliClient::new(account) {
                if let Ok(res) = client
                        .get(WBI)
                        .header("Referer", "https://www.bilibili.com")
                        .send()
                        .await
                {
                        if let Ok(data) = res.json::<Value>().await {
                                if data["code"].as_i64().unwrap_or(-1) == 0 {
                                        let user_data = &data["data"];
                                        let uname = user_data["uname"]
                                                .as_str()
                                                .unwrap_or("未知用户")
                                                .to_string();
                                        let is_login =
                                                user_data["isLogin"].as_bool().unwrap_or(true);
                                        let vip_type =
                                                match user_data["vipType"].as_i64().unwrap_or(0) {
                                                        | 2 => "年度大会员".to_string(),
                                                        | 1 => "月度大会员".to_string(),
                                                        | _ => "普通用户".to_string(),
                                                };
                                        let vip_status_desc = user_data["vip_label"]["text"]
                                                .as_str()
                                                .unwrap_or(&vip_type)
                                                .to_string();

                                        let _ = tx.send(AppEvent::UserProfileLoaded(UserProfile {
                                                uid,
                                                uname,
                                                is_login,
                                                vip_type,
                                                vip_status_desc,
                                                exp_time: None,
                                                sessdata_preview,
                                        }));
                                        return;
                                }
                        }
                }
        }

        let _ = tx.send(AppEvent::UserProfileLoaded(UserProfile {
                uid,
                uname: "B站用户".to_string(),
                is_login: !account.is_expired(),
                vip_type: "已登录".to_string(),
                vip_status_desc: "正常".to_string(),
                exp_time: None,
                sessdata_preview,
        }));
}

pub async fn start_qr_login_flow(tx: UnboundedSender<AppEvent>) {
        let client = match Client::builder().user_agent(UA).build() {
                | Ok(c) => c,
                | Err(e) => {
                        let _ = tx.send(AppEvent::QrStatusUpdate(QrLoginStatus::Error(format!(
                                "创建网络客户端失败: {}",
                                e
                        ))));
                        return;
                },
        };

        let _ = tx.send(AppEvent::QrStatusUpdate(QrLoginStatus::Generating));

        let resp_result = client.get(LOGIN).send().await;
        let resp_json: Option<Value> = match resp_result {
                | Ok(res) => res.json().await.ok(),
                | Err(e) => {
                        let _ = tx.send(AppEvent::QrStatusUpdate(QrLoginStatus::Error(format!(
                                "请求二维码接口失败: {}",
                                e
                        ))));
                        return;
                },
        };

        let (url, qrcode_key) = match resp_json {
                | Some(resp) => {
                        let url = resp["data"]["url"].as_str().map(|s| s.to_string());
                        let key = resp["data"]["qrcode_key"].as_str().map(|s| s.to_string());
                        if let (Some(u), Some(k)) = (url, key) {
                                (u, k)
                        } else {
                                let _ = tx.send(AppEvent::QrStatusUpdate(QrLoginStatus::Error(
                                        "无法解析二维码数据".into(),
                                )));
                                return;
                        }
                },
                | None => {
                        let _ = tx.send(AppEvent::QrStatusUpdate(QrLoginStatus::Error(
                                "解析二维码JSON失败".into(),
                        )));
                        return;
                },
        };

        let code = match QrCode::new(url.as_bytes()) {
                | Ok(c) => c,
                | Err(e) => {
                        let _ = tx.send(AppEvent::QrStatusUpdate(QrLoginStatus::Error(format!(
                                "生成二维码矩阵失败: {}",
                                e
                        ))));
                        return;
                },
        };

        let qr_text = code.render::<unicode::Dense1x2>().quiet_zone(false).build();
        let _ = tx.send(AppEvent::QrStatusUpdate(QrLoginStatus::WaitingScan {
                qr_text,
                qr_key: qrcode_key.clone(),
        }));

        loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                let resp = match client
                        .get(VALIDATE_QRCODE)
                        .query(&[("qrcode_key", &qrcode_key)])
                        .send()
                        .await
                {
                        | Ok(r) => r,
                        | Err(_) => continue,
                };

                let cookies: Vec<String> = resp
                        .headers()
                        .get_all("set-cookie")
                        .iter()
                        .filter_map(|v| v.to_str().ok().map(|s| s.to_string()))
                        .collect();

                let resp_value: Value = match resp.json().await {
                        | Ok(v) => v,
                        | Err(_) => continue,
                };

                match resp_value["data"]["code"].as_i64().unwrap_or(-1) {
                        | 0 => {
                                let user_id = match cookies
                                        .iter()
                                        .find(|c| c.starts_with("DedeUserID="))
                                        .and_then(|c| c.split(';').next())
                                        .and_then(|c| c.split('=').nth(1))
                                {
                                        | Some(id) => id.to_string(),
                                        | None => {
                                                let _ = tx.send(AppEvent::QrStatusUpdate(
                                                        QrLoginStatus::Error(
                                                                "未找到 DedeUserID".into(),
                                                        ),
                                                ));
                                                return;
                                        },
                                };

                                let sessdata_cookie =
                                        match cookies.iter().find(|c| c.starts_with("SESSDATA=")) {
                                                | Some(c) => c,
                                                | None => {
                                                        let _ = tx.send(AppEvent::QrStatusUpdate(
                                                                QrLoginStatus::Error(
                                                                        "未找到 SESSDATA".into(),
                                                                ),
                                                        ));
                                                        return;
                                                },
                                        };

                                let sessdata = match sessdata_cookie
                                        .split(';')
                                        .next()
                                        .and_then(|s| s.split('=').nth(1))
                                {
                                        | Some(sd) => sd.to_string(),
                                        | None => {
                                                let _ = tx.send(AppEvent::QrStatusUpdate(
                                                        QrLoginStatus::Error(
                                                                "SESSDATA 格式异常".into(),
                                                        ),
                                                ));
                                                return;
                                        },
                                };

                                let expire_time = sessdata_cookie
                                        .split("Expires=")
                                        .nth(1)
                                        .and_then(|s| s.split(';').next())
                                        .and_then(|s| httpdate::parse_http_date(s.trim()).ok())
                                        .unwrap_or(SystemTime::now());

                                let account = Account::new(user_id, expire_time, sessdata);
                                let _ = tx.send(AppEvent::QrStatusUpdate(QrLoginStatus::Success));
                                let _ = tx.send(AppEvent::LoginSuccess(account));
                                return;
                        },
                        | 86101 => {},
                        | 86090 => {
                                let _ = tx.send(AppEvent::QrStatusUpdate(
                                        QrLoginStatus::ScannedWaitingConfirm,
                                ));
                        },
                        | 86038 => {
                                let _ = tx.send(AppEvent::QrStatusUpdate(QrLoginStatus::Expired));
                                return;
                        },
                        | _ => {},
                }
        }
}

pub async fn parse_video_url(
        url: String,
        account: Option<Account>,
        tx: UnboundedSender<AppEvent>,
) {
        let bili_client = match account {
                | Some(ref acc) => BiliClient::new(acc).ok(),
                | None => None,
        };

        // 首先尝试是否是媒体合集/收藏夹链接 (含有 ml...)
        if let Ok(ml_id) = util::extract_media_id(&url) {
                match get_basic_collection_info(&ml_id, 1, 20, bili_client.as_ref()).await {
                        | Ok(cur) => match cur.get_data() {
                                | Ok(data) => {
                                        let title = format!("【收藏夹/合集】{}", data.info.title);
                                        let owner = data.info.upper.name.clone();
                                        let count = data.info.media_count;
                                        let pic = data.info.cover.clone();
                                        let desc = data.info.intro.clone().unwrap_or_default();

                                        let _ = tx.send(AppEvent::VideoInfoParsed(Ok(
                                                VideoInfoPreview {
                                                        bvid: format!("ml{}", ml_id),
                                                        title,
                                                        owner,
                                                        duration_secs: 0,
                                                        pic,
                                                        desc,
                                                        is_collection: true,
                                                        media_count: Some(count),
                                                },
                                        )));
                                        return;
                                },
                                | Err(e) => {
                                        let _ = tx.send(AppEvent::VideoInfoParsed(Err(format!(
                                                "解析收藏夹数据失败: {}",
                                                e
                                        ))));
                                        return;
                                },
                        },
                        | Err(e) => {
                                let _ = tx.send(AppEvent::VideoInfoParsed(Err(format!(
                                        "请求收藏夹/合集信息失败: {}",
                                        e
                                ))));
                                return;
                        },
                }
        }

        // 否则按普通单视频 BV 解析
        let bvid = match util::extract_bv_id(&url) {
                | Ok(id) => id,
                | Err(e) => {
                        let _ = tx.send(AppEvent::VideoInfoParsed(Err(format!(
                                "提取 BV 号 / 合集 ID 失败: {}",
                                e
                        ))));
                        return;
                },
        };

        let client = match Client::builder().user_agent(UA).build() {
                | Ok(c) => c,
                | Err(e) => {
                        let _ = tx.send(AppEvent::VideoInfoParsed(Err(format!(
                                "网络客户端初始化失败: {}",
                                e
                        ))));
                        return;
                },
        };

        let request = if let Some(ref bc) = bili_client {
                bc.get(VIDEO_INFO)
        } else {
                client.get(VIDEO_INFO)
        };

        let resp_result = request
                .header("Referer", "https://www.bilibili.com")
                .query(&[("bvid", &bvid)])
                .send()
                .await;

        match resp_result {
                | Ok(resp) => {
                        if let Ok(json) = resp.json::<Value>().await {
                                if json["code"].as_i64().unwrap_or(-1) == 0 {
                                        let data = &json["data"];
                                        let title = data["title"]
                                                .as_str()
                                                .unwrap_or("未知标题")
                                                .to_string();
                                        let owner = data["owner"]["name"]
                                                .as_str()
                                                .unwrap_or("未知UP主")
                                                .to_string();
                                        let duration_secs = data["duration"].as_u64().unwrap_or(0);
                                        let pic = data["pic"].as_str().unwrap_or("").to_string();
                                        let desc = data["desc"]
                                                .as_str()
                                                .unwrap_or("")
                                                .chars()
                                                .take(120)
                                                .collect();

                                        let _ = tx.send(AppEvent::VideoInfoParsed(Ok(
                                                VideoInfoPreview {
                                                        bvid,
                                                        title,
                                                        owner,
                                                        duration_secs,
                                                        pic,
                                                        desc,
                                                        is_collection: false,
                                                        media_count: None,
                                                },
                                        )));
                                } else {
                                        let msg = json["message"]
                                                .as_str()
                                                .unwrap_or("视频解析返回异常");
                                        let _ = tx.send(AppEvent::VideoInfoParsed(Err(
                                                msg.to_string()
                                        )));
                                }
                        } else {
                                let _ = tx.send(AppEvent::VideoInfoParsed(Err(
                                        "视频数据解析失败".to_string()
                                )));
                        }
                },
                | Err(e) => {
                        let _ = tx.send(AppEvent::VideoInfoParsed(Err(format!(
                                "请求视频信息失败: {}",
                                e
                        ))));
                },
        }
}

pub async fn execute_download(
        task_id: usize,
        url: String,
        mode: DownloadMode,
        output_dir: PathBuf,
        ffmpeg_path: Option<PathBuf>,
        video_quality: ybd_core::model::quality::VideoQuality,
        audio_quality: ybd_core::model::quality::AudioQuality,
        video_encode: VideoEncode,
        batch: bool,
        account: Option<Account>,
        tx: UnboundedSender<AppEvent>,
) {
        if let Err(e) = tokio::fs::create_dir_all(&output_dir).await {
                let _ = tx.send(AppEvent::TaskFailed {
                        task_id,
                        error: format!("创建输出目录失败: {}", e),
                });
                return;
        }

        let acc = match account {
                | Some(a) => {
                        if a.is_expired() {
                                let _ = tx.send(AppEvent::TaskFailed {
                                        task_id,
                                        error: "账号登录凭据已过期，请先按 1 进入个人中心重新扫码登录".to_string(),
                                });
                                return;
                        }
                        a
                },
                | None => {
                        let _ = tx.send(AppEvent::TaskFailed {
                                task_id,
                                error: "未登录账号！请先按 1 进入个人中心按 r 扫码登录后再下载"
                                        .to_string(),
                        });
                        return;
                },
        };

        let bili_client = match BiliClient::new(&acc) {
                | Ok(bc) => bc,
                | Err(e) => {
                        let _ = tx.send(AppEvent::TaskFailed {
                                task_id,
                                error: format!("客户端初始化失败: {}", e),
                        });
                        return;
                },
        };

        // 处理批量/合集/收藏夹下载逻辑
        if batch || util::extract_media_id(&url).is_ok() {
                let ml_id = match util::extract_media_id(&url) {
                        | Ok(id) => id,
                        | Err(_) => "".to_string(),
                };

                if !ml_id.is_empty() {
                        let mut bv_ids = Vec::<String>::new();
                        let mut pn = 1usize;
                        loop {
                                let resp = match get_basic_collection_info(
                                        &ml_id,
                                        pn,
                                        20,
                                        Some(&bili_client),
                                )
                                .await
                                {
                                        | Ok(r) => r,
                                        | Err(e) => {
                                                let _ = tx.send(AppEvent::TaskFailed {
                                                        task_id,
                                                        error: format!(
                                                                "获取合集视频列表失败: {}",
                                                                e
                                                        ),
                                                });
                                                return;
                                        },
                                };

                                let data = match resp.get_data() {
                                        | Ok(d) => d,
                                        | Err(e) => {
                                                let _ = tx.send(AppEvent::TaskFailed {
                                                        task_id,
                                                        error: format!("解析合集数据失败: {}", e),
                                                });
                                                return;
                                        },
                                };

                                let bvids: Vec<String> =
                                        data.medias.iter().map(|v| v.bvid.clone()).collect();
                                bv_ids.extend(bvids);
                                if !data.has_more {
                                        break;
                                }
                                pn += 1;
                        }

                        if bv_ids.is_empty() {
                                let _ = tx.send(AppEvent::TaskFailed {
                                        task_id,
                                        error: "合集内没有找到有效视频".into(),
                                });
                                return;
                        }

                        let total_count = bv_ids.len();
                        let bc = Arc::new(bili_client);
                        let base_path = Arc::new(output_dir);
                        let ffmpeg = Arc::new(ffmpeg_path);
                        let semaphore = Arc::new(Semaphore::new(2));
                        let mut handlers = Vec::new();

                        for (idx, bv_id) in bv_ids.into_iter().enumerate() {
                                let bc = Arc::clone(&bc);
                                let base_path = Arc::clone(&base_path);
                                let sp = Arc::clone(&semaphore);
                                let ffmpeg_s = Arc::clone(&ffmpeg);
                                let tx_inner = tx.clone();
                                let sub_id = task_id * 1000 + idx + 1;

                                let jh = tokio::spawn(async move {
                                        let _permit = match sp.acquire().await {
                                                | Ok(p) => p,
                                                | Err(_) => return,
                                        };

                                        let (title, pic, _) = match actuator::get_basic_video_info(
                                                &bv_id,
                                                Some(&bc),
                                        )
                                        .await
                                        {
                                                | Ok(info) => info,
                                                | Err(e) => {
                                                        let _ = tx_inner.send(AppEvent::TaskFailed {
                                                                task_id: sub_id,
                                                                error: format!(
                                                                        "获取信息失败 BV {}: {}",
                                                                        bv_id, e
                                                                ),
                                                        });
                                                        return;
                                                },
                                        };

                                        let sanitized = sanitize_filename::sanitize(&title);
                                        let _ = tx_inner.send(AppEvent::TaskCreated {
                                                task_id: sub_id,
                                                bvid: bv_id.clone(),
                                                title: format!(
                                                        "[{}/{}] {}",
                                                        idx + 1,
                                                        total_count,
                                                        title
                                                ),
                                                mode,
                                        });

                                        let tx_v = tx_inner.clone();
                                        let tx_a = tx_inner.clone();

                                        match mode {
                                                | DownloadMode::Cover => {
                                                        let client = match Client::builder()
                                                                .user_agent(UA)
                                                                .build()
                                                        {
                                                                | Ok(c) => c,
                                                                | Err(_) => return,
                                                        };
                                                        let cover_path = base_path
                                                                .join(format!("{}.png", sanitized));
                                                        if !cover_path.exists() {
                                                                let _ = actuator::download_cover(
                                                                        &client,
                                                                        &pic,
                                                                        &cover_path,
                                                                )
                                                                .await;
                                                        }
                                                        let _ = tx_inner.send(
                                                                AppEvent::TaskCompleted {
                                                                        task_id: sub_id,
                                                                },
                                                        );
                                                },
                                                | DownloadMode::Audio => {
                                                        let audio_path = base_path
                                                                .join(format!("{}.m4a", sanitized));
                                                        if audio_path.exists() {
                                                                let _ = tx_inner.send(
                                                                        AppEvent::TaskCompleted {
                                                                                task_id: sub_id,
                                                                        },
                                                                );
                                                                return;
                                                        }

                                                        if let Ok(pur) =
                                                                PlayUrlResponse::new(&bc, &bv_id)
                                                                        .await
                                                        {
                                                                let opt = DownloadOption::builder()
                                                                        .audio_path(&audio_path)
                                                                        .audio_quality(audio_quality)
                                                                        .on_audio_progress(Arc::new(move |d, t| {
                                                                                let _ = tx_a.send(AppEvent::TaskAudioProgress {
                                                                                        task_id: sub_id,
                                                                                        downloaded: d,
                                                                                        total: t,
                                                                                });
                                                                        }))
                                                                        .build();
                                                                let res = actuator::download_audio(
                                                                        &bc, &pur, &opt,
                                                                )
                                                                .await;
                                                                if res.is_ok() {
                                                                        if !util::check_cover_box(
                                                                                &audio_path,
                                                                        )
                                                                        .unwrap_or(true)
                                                                        {
                                                                                if let Ok(cb) = util::download_cover_bytes(&bc, &pic).await {
                                                                                        let _ = util::add_cover_box(&audio_path, cb);
                                                                                }
                                                                        }
                                                                        let _ = tx_inner.send(AppEvent::TaskCompleted { task_id: sub_id });
                                                                } else if let Err(e) = res {
                                                                        let _ = tx_inner.send(AppEvent::TaskFailed {
                                                                                task_id: sub_id,
                                                                                error: e.to_string(),
                                                                        });
                                                                }
                                                        }
                                                },
                                                | DownloadMode::Video => {
                                                        let video_path = base_path
                                                                .join(format!("{}.mp4", sanitized));
                                                        let v_tmp = video_path
                                                                .with_extension("video.tmp");
                                                        let a_tmp = video_path
                                                                .with_extension("audio.tmp");
                                                        if video_path.exists() {
                                                                let _ = tx_inner.send(
                                                                        AppEvent::TaskCompleted {
                                                                                task_id: sub_id,
                                                                        },
                                                                );
                                                                return;
                                                        }

                                                        if let Ok(pur) =
                                                                PlayUrlResponse::new(&bc, &bv_id)
                                                                        .await
                                                        {
                                                                let mut b = DownloadOption::builder()
                                                                        .video_encode(video_encode)
                                                                        .video_quality(video_quality)
                                                                        .audio_quality(audio_quality)
                                                                        .video_path(&v_tmp)
                                                                        .audio_path(&a_tmp)
                                                                        .output(&video_path)
                                                                        .on_video_progress(Arc::new(move |d, t| {
                                                                                let _ = tx_v.send(AppEvent::TaskVideoProgress {
                                                                                        task_id: sub_id,
                                                                                        downloaded: d,
                                                                                        total: t,
                                                                                });
                                                                        }))
                                                                        .on_audio_progress(Arc::new(move |d, t| {
                                                                                let _ = tx_a.send(AppEvent::TaskAudioProgress {
                                                                                        task_id: sub_id,
                                                                                        downloaded: d,
                                                                                        total: t,
                                                                                });
                                                                        }));
                                                                if let Some(ref ff) = *ffmpeg_s {
                                                                        b = b.ffmpeg_path(ff);
                                                                }
                                                                let opt = b.build();
                                                                let res = actuator::download_video(
                                                                        &bc, &pur, &opt,
                                                                )
                                                                .await;
                                                                if res.is_ok() {
                                                                        let _ = tx_inner.send(AppEvent::TaskCompleted { task_id: sub_id });
                                                                } else if let Err(e) = res {
                                                                        let _ = tx_inner.send(AppEvent::TaskFailed {
                                                                                task_id: sub_id,
                                                                                error: e.to_string(),
                                                                        });
                                                                }
                                                        }
                                                },
                                        }
                                });
                                handlers.push(jh);
                        }

                        let _ = future::join_all(handlers).await;
                        let _ = tx.send(AppEvent::TaskCompleted { task_id });
                        return;
                }
        }

        // 单视频下载逻辑
        let bv_id = match util::extract_bv_id(&url) {
                | Ok(id) => id,
                | Err(e) => {
                        let _ = tx.send(AppEvent::TaskFailed {
                                task_id,
                                error: format!("提取BV号失败: {}", e),
                        });
                        return;
                },
        };

        let (title, pic, _) = match actuator::get_basic_video_info(&bv_id, Some(&bili_client)).await
        {
                | Ok(info) => info,
                | Err(e) => {
                        let _ = tx.send(AppEvent::TaskFailed {
                                task_id,
                                error: format!("获取视频基础信息失败: {}", e),
                        });
                        return;
                },
        };

        let sanitized_title = sanitize_filename::sanitize(&title);

        match mode {
                | DownloadMode::Cover => {
                        let client = match Client::builder().user_agent(UA).build() {
                                | Ok(c) => c,
                                | Err(e) => {
                                        let _ = tx.send(AppEvent::TaskFailed {
                                                task_id,
                                                error: format!("网络客户端异常: {}", e),
                                        });
                                        return;
                                },
                        };
                        let cover_path = output_dir.join(format!("{}.png", sanitized_title));
                        match actuator::download_cover(&client, &pic, &cover_path).await {
                                | Ok(_) => {
                                        let _ = tx.send(AppEvent::TaskCompleted { task_id });
                                },
                                | Err(e) => {
                                        let _ = tx.send(AppEvent::TaskFailed {
                                                task_id,
                                                error: format!("下载封面失败: {}", e),
                                        });
                                },
                        }
                },
                | DownloadMode::Audio => {
                        let audio_path = output_dir.join(format!("{}.m4a", sanitized_title));
                        let pur = match PlayUrlResponse::new(&bili_client, &bv_id).await {
                                | Ok(p) => p,
                                | Err(e) => {
                                        let _ = tx.send(AppEvent::TaskFailed {
                                                task_id,
                                                error: format!(
                                                        "获取播放流地址失败 (请检查登录状态): {}",
                                                        e
                                                ),
                                        });
                                        return;
                                },
                        };

                        let tx_clone = tx.clone();
                        let builder = DownloadOption::builder()
                                .audio_path(&audio_path)
                                .audio_quality(audio_quality)
                                .on_audio_progress(Arc::new(move |downloaded, total| {
                                        let _ = tx_clone.send(AppEvent::TaskAudioProgress {
                                                task_id,
                                                downloaded,
                                                total,
                                        });
                                }));

                        let option = builder.build();
                        match actuator::download_audio(&bili_client, &pur, &option).await {
                                | Ok(_) => {
                                        if !util::check_cover_box(&audio_path).unwrap_or(true) {
                                                if let Ok(cover_bytes) = util::download_cover_bytes(
                                                        &bili_client,
                                                        &pic,
                                                )
                                                .await
                                                {
                                                        let _ = util::add_cover_box(
                                                                &audio_path,
                                                                cover_bytes,
                                                        );
                                                }
                                        }
                                        let _ = tx.send(AppEvent::TaskCompleted { task_id });
                                },
                                | Err(e) => {
                                        let _ = tx.send(AppEvent::TaskFailed {
                                                task_id,
                                                error: format!("下载音频失败: {}", e),
                                        });
                                },
                        }
                },
                | DownloadMode::Video => {
                        let video_path = output_dir.join(format!("{}.mp4", sanitized_title));
                        let video_tmp = video_path.with_extension("video.tmp");
                        let audio_tmp = video_path.with_extension("audio.tmp");

                        let pur = match PlayUrlResponse::new(&bili_client, &bv_id).await {
                                | Ok(p) => p,
                                | Err(e) => {
                                        let _ = tx.send(AppEvent::TaskFailed {
                                                task_id,
                                                error: format!(
                                                        "获取播放地址失败 (请检查登录状态): {}",
                                                        e
                                                ),
                                        });
                                        return;
                                },
                        };

                        let tx_v = tx.clone();
                        let tx_a = tx.clone();

                        let mut builder = DownloadOption::builder()
                                .video_encode(video_encode)
                                .video_quality(video_quality)
                                .audio_quality(audio_quality)
                                .video_path(&video_tmp)
                                .audio_path(&audio_tmp)
                                .output(&video_path)
                                .on_video_progress(Arc::new(move |downloaded, total| {
                                        let _ = tx_v.send(AppEvent::TaskVideoProgress {
                                                task_id,
                                                downloaded,
                                                total,
                                        });
                                }))
                                .on_audio_progress(Arc::new(move |downloaded, total| {
                                        let _ = tx_a.send(AppEvent::TaskAudioProgress {
                                                task_id,
                                                downloaded,
                                                total,
                                        });
                                }));

                        if let Some(ref ffmpeg) = ffmpeg_path {
                                builder = builder.ffmpeg_path(ffmpeg);
                        }

                        let option = builder.build();

                        match actuator::download_video(&bili_client, &pur, &option).await {
                                | Ok(_) => {
                                        let _ = tx.send(AppEvent::TaskCompleted { task_id });
                                },
                                | Err(e) => {
                                        let _ = tx.send(AppEvent::TaskFailed {
                                                task_id,
                                                error: format!("下载视频失败: {}", e),
                                        });
                                },
                        }
                },
        }
}
