use std::error::Error;

use reqwest::Client;
use serde_json::Value;
use yumi_bilibili_download::{
    actuator::{get_basic_video_info, get_wbi_keys},
    login,
    model::param::VideoRequestParamBuilder,
    url::{UA, VEDIO_DOWNLOAD_URL},
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::builder().user_agent(UA).build()?;
    let account = login::get_account(&client).await?;
    let sessdata = account.get_sessdata();
    let bvid = "BV1jp421Z7jS";
    let (_, cid) = get_basic_video_info(&client, bvid, sessdata).await?;
    let (img_key, sub_key) = get_wbi_keys(&client, sessdata).await?;
    let vrp = VideoRequestParamBuilder::new(bvid, cid).build(img_key, sub_key)?;
    let url = format!("{}?{}", VEDIO_DOWNLOAD_URL, vrp.to_query_string());
    let resp: Value = client
        .get(url)
        .header("Cookie", format!("SESSDATA={}", sessdata))
        .header("Referer", "https://www.bilibili.com")
        .send()
        .await?
        .json()
        .await?;
    let json_str = serde_json::to_string_pretty(&resp)?;
    tokio::fs::write("resp.json", json_str).await?;
    println!("{resp:#?}");
    Ok(())
}
