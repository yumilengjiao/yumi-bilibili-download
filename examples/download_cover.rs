use std::{error::Error, path::Path};

use reqwest::Client;
use tokio::fs;
use yumi_bilibili_download::{
    actuator::{download_cover, get_basic_video_info},
    url::UA,
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::builder().user_agent(UA).build()?;
    let bvid = "BV1jp421Z7jS";
    let (title, url, _) = get_basic_video_info(bvid).await?;
    let title = sanitize_filename::sanitize(&title);
    fs::create_dir_all("output").await?;
    download_cover(&client, &url, Path::new(&format!("output/{}.png", &title))).await?;
    println!("The cover is outputed to output/{}.png", title);
    Ok(())
}
