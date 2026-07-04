use std::error::Error;

use yumi_bilibili_download::{actuator::get_basic_collection_info, util::extract_media_id};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
        let mlid = extract_media_id(
                "https://www.bilibili.com/list/ml2408095182?spm_id_from=333.1007.0.0&oid=113601096516735&bvid=BV12KifY6EKb",
        )?;
        let resp = get_basic_collection_info(&mlid, 1, 20, None).await?;

        let json_str = serde_json::to_string_pretty(&resp)?;
        tokio::fs::create_dir_all("output").await?;
        tokio::fs::write("output/video_example.json", json_str).await?;
        println!("The res is outputed to output/collection_example.json");
        Ok(())
}
