use std::{error::Error, path::Path};

use reqwest::Client;
use tokio::fs;
use yumi_bilibili_download::{
    actuator::{download_audio, get_basic_video_info, get_wbi_keys},
    login,
    model::{param::VideoRequestParamBuilder, vedio::PlayUrlResponse},
    url::{UA, VEDIO_DOWNLOAD_URL},
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::builder().user_agent(UA).build()?;
    let account = login::get_account(&client).await?;
    let sessdata = account.get_sessdata();
    let bvid = "BV1JvPMedE5E";
    let (_, _, cid) = get_basic_video_info(&client, bvid).await?;
    let (img_key, sub_key) = get_wbi_keys(&client, sessdata).await?;
    let vrp = VideoRequestParamBuilder::new(bvid, cid).build(img_key, sub_key)?;
    let url = format!("{}?{}", VEDIO_DOWNLOAD_URL, vrp.to_query_string());
    let resp: PlayUrlResponse = client
        .get(url)
        .header("Cookie", format!("SESSDATA={}", sessdata))
        .header("Referer", "https://www.bilibili.com")
        .send()
        .await?
        .json()
        .await?;
    download_audio(
        &client,
        &resp.get_data()?,
        Path::new("output/audio_example.m4a"),
        sessdata,
    )
    .await?;
    fs::create_dir_all("output").await?;
    println!("The res is outputed to output/audio_example.m4a");
    Ok(())
}
