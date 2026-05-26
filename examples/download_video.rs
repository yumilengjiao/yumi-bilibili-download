use std::{error::Error, path::Path};
use tokio::fs;
use yumi_bilibili_download::{actuator, client::BiliClient, login, model::video::PlayUrlResponse};
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let account = login::get_account().await?;
    let biliclient = BiliClient::new(&account)?;
    let bvid = "BV1JvPMedE5E";

    let resp = PlayUrlResponse::new(&biliclient, bvid).await?;
    fs::create_dir_all("output").await?;
    actuator::download_video(
        &biliclient,
        &resp,
        None,
        None,
        None,
        None,
        Path::new("output/audio_example.m4a"),
        Path::new("output/video_example.mp4"),
        Path::new("output/video_sample.mp4"),
    )
    .await?;
    println!("The video is outputed to output/video_sample.mp4");
    Ok(())
}
