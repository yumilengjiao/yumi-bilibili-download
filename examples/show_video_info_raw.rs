use std::error::Error;

use serde_json::Value;
use yumi_bilibili_download::{
        client::BiliClient, login, model::param::VideoRequestParamBuilder, url::VIDEO_DOWNLOAD_URL,
        util::extract_bv_id,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
        let account = login::get_account().await?;
        let bili_client = BiliClient::new(&account)?;
        let bvid = extract_bv_id(
                "https://www.bilibili.com/video/BV17W411e7VL/?spm_id_from=333.337.search-card.all.click&vd_source=8bd37d9aaf81178ae3e80214a1c8e367",
        )?;
        let vrp = VideoRequestParamBuilder::new(&bvid, &bili_client)
                .await?
                .build(&bili_client)
                .await?;
        let url = format!("{}?{}", VIDEO_DOWNLOAD_URL, vrp.to_query_string());
        let resp: Value = bili_client.get(&url).send().await?.json().await?;

        let json_str = serde_json::to_string_pretty(&resp)?;
        tokio::fs::create_dir_all("output").await?;
        tokio::fs::write("output/video_example.json", json_str).await?;
        println!("The res is outputed to output/video_example.json");
        Ok(())
}
