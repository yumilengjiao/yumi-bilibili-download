use std::error::Error;

use yumi_bilibili_download::{
    client::BiliClient, login, model::video::PlayUrlResponse, util::extract_bv_id,
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let account = login::get_account().await?;
    let bili_client = BiliClient::new(&account)?;
    let bvid = extract_bv_id(
        "https://www.bilibili.com/video/BV12KifY6EKb/?spm_id_from=333.337.search-card.all.click&vd_source=8bd37d9aaf81178ae3e80214a1c8e367",
    )?;
    let resp = PlayUrlResponse::new(&bili_client, &bvid).await?;

    let json_str = serde_json::to_string_pretty(&resp)?;
    tokio::fs::create_dir_all("output").await?;
    tokio::fs::write("output/video_example.json", json_str).await?;
    println!("The res is outputed to output/video_example.json");
    Ok(())
}
