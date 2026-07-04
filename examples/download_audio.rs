use std::{error::Error, path::Path};

use tokio::fs;
use yumi_bilibili_download::{
        actuator,
        client::BiliClient,
        login,
        model::{download::DownloadOption, video::PlayUrlResponse},
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
        let account = login::get_account().await?;
        let biliclient = BiliClient::new(&account)?;
        let bvid = "BV1JvPMedE5E";

        let resp = PlayUrlResponse::new(&biliclient, bvid).await?;
        fs::create_dir_all("output").await?;
        let option = DownloadOption::builder()
                .audio_path(Path::new("output/audio_example.m4a"))
                .build();
        actuator::download_audio(&biliclient, &resp, &option).await?;
        println!("The res is outputed to output/audio_example.m4a");
        Ok(())
}
