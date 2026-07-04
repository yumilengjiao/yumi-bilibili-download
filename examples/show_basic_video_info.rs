use std::error::Error;

use serde_json::Value;
use tokio::fs;
use yumi_bilibili_download::{client::BiliClient, login, url::VIDEO_INFO};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
        let account = login::get_account().await?;
        let biliclient = BiliClient::new(&account)?;
        let resp: Value = biliclient
                .get(VIDEO_INFO)
                .header("Referer", "https://www.bilibili.com")
                .query(&[("bvid", "BV1bzLy6EEbn")])
                .send()
                .await?
                .json()
                .await?;
        fs::create_dir_all("./output").await?;
        let s = serde_json::to_string_pretty(&resp)?;
        fs::write("output/basic_video_info.json", s).await?;
        println!("结果输出至output/basic_video_info.json");
        Ok(())
}
