use std::error::Error;

use yumi_bilibili_download::{client::BiliClient, login, model::vedio::PlayUrlResponse};
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let account = login::get_account().await?;
    let bili_client = BiliClient::new(&account)?;
    let bvid = "BV1JvPMedE5E";
    let resp = PlayUrlResponse::new(&bili_client, bvid).await?;

    let json_str = serde_json::to_string_pretty(&resp)?;
    tokio::fs::create_dir_all("output").await?;
    tokio::fs::write("output/vedio_example.json", json_str).await?;
    println!("The res is outputed to output/vedio_example.json");
    Ok(())
}
