use std::{error::Error, path::Path};
use tokio::fs;
use yumi_bilibili_download::{
        actuator,
        client::BiliClient,
        login,
        model::{download::DownloadOption, quality::VideoEncode, video::PlayUrlResponse},
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
        let account = login::get_account().await?;
        let biliclient = BiliClient::new(&account)?;
        let bvid = "BV1JvPMedE5E";

        let resp = PlayUrlResponse::new(&biliclient, bvid).await?;
        fs::create_dir_all("output").await?;
        let option = DownloadOption::builder()
                // potplayer低于v1.7.21759,尝试GPU解码av1编码格式时会有黑屏问题,这里用avc编码
                .video_encode(VideoEncode::AVC)
                .audio_path(Path::new("output/audio_example.m4a"))
                .video_path(Path::new("output/video_example.mp4"))
                .output(Path::new("output/video_sample.mp4"))
                .build();
        actuator::download_video(&biliclient, &resp, &option).await?;
        println!("The video is outputed to output/video_sample.mp4");
        Ok(())
}
