//! 下载音频并写入完整元数据（标题/UP 主/简介/年份/分P/标签/封面）
//!
//! 运行：`cargo run -p ybd-core --example download_audio_with_metadata`
//!
//! 需要先登录（`ybd login`），生成的文件可直接被各类播放器识别出
//! “歌手/专辑/封面/年份/分P”。

use std::{error::Error, path::Path};

use tokio::fs;
use ybd_core::{
        actuator,
        client::BiliClient,
        login,
        model::{download::DownloadOption, video::PlayUrlResponse},
        util,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
        let account = login::get_account().await?;
        let biliclient = BiliClient::new(&account)?;
        let bvid = "BV1JvPMedE5E";

        // 1. 拉取视频详细信息（标题、UP 主、简介、发布时间、分P）
        let detail = actuator::get_video_detail(bvid, Some(&biliclient)).await?;
        println!("标题: {}", detail.title);
        println!("UP 主: {}", detail.owner.name);
        println!("分区: {}", detail.tname);

        fs::create_dir_all("output").await?;
        let audio_path = Path::new("output/audio_with_metadata.m4a");

        // 2. 下载音频
        let resp = PlayUrlResponse::new(&biliclient, bvid).await?;
        let option = DownloadOption::builder().audio_path(audio_path).build();
        actuator::download_audio(&biliclient, &resp, &option).await?;

        // 3. 先看一下将要写入的元数据
        let meta = detail.media_meta(Some("我的收藏夹"), Some(detail.cid));
        println!("{:#?}", meta);

        // 4. 写入元数据 + 封面（已写入则自动跳过）
        actuator::embed_metadata(
                &biliclient,
                audio_path,
                &detail,
                Some("我的收藏夹"),
                Some(detail.cid),
        )
        .await?;

        // 5. 验证结果
        let tag = mp4ameta::Tag::read_from_path(audio_path)?;
        println!("----------------------------------------");
        println!("title:       {:?}", tag.title());
        println!("artist:      {:?}", tag.artist());
        println!("album:       {:?}", tag.album());
        println!("genre:       {:?}", tag.genre());
        println!("year:        {:?}", tag.year());
        println!("track:       {:?}", tag.track());
        println!("comment:     {:?}", tag.comment());
        println!("description: {:?}", tag.description());
        println!("keywords:    {:?}", tag.keywords().collect::<Vec<_>>());
        println!("encoder:     {:?}", tag.encoder());
        println!("cover:       {:?}", util::check_cover_box(audio_path)?);

        Ok(())
}
