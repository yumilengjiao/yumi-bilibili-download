use std::path::Path;

use mp4ameta::{Img, Tag};
use regex::Regex;

use crate::{
        client::BiliClient,
        error::{Error, Result},
};

/// 从链接提取bvid
///
/// * `bv_id`: 视频url
pub fn extract_bv_id(input: &str) -> Result<String> {
        let regex = Regex::new(r"BV[a-z0-9A-Z]+").expect("不正确的正则表达式");
        regex.find(input)
                .map(|m| m.as_str().to_string())
                .ok_or(Error::Normal("无法从输入中提取bv号".into()))
}

// eg. https://www.bilibili.com/list/ml2408095182?spm_id_from=333.1007.0.0&oid=115724639078994&bvid=BV1S4qWBkEGk
pub fn extract_media_id(input: &str) -> Result<String> {
        let regex = Regex::new(r"/ml(\d+)\?").expect("不正确的正则表达式");
        regex.captures(input)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str().to_string())
                .ok_or(Error::Normal("无法从输入中提取mlid号".into()))
}

/// 直接下载封面到内存
///
/// * `client`: 客户端
/// * `cover_url`: 链接
pub async fn download_cover_bytes(
        client: &BiliClient,
        cover_url: &str,
) -> Result<Vec<u8>> {
        let response = client.get(cover_url).send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
}

pub fn check_cover_box(file_path: &Path) -> Result<bool> {
        let tag = Tag::read_from_path(file_path)?;
        Ok(tag.artwork().is_some())
}

pub fn add_cover_box(
        file_path: &Path,
        image_data: Vec<u8>,
) -> Result<()> {
        let fmt = detect_image_format(&image_data);
        let mut tag = Tag::read_from_path(file_path)?;
        let image = Img::new(fmt, image_data);
        tag.set_artwork(image);
        tag.write_to_path(file_path)?;
        Ok(())
}

fn detect_image_format(bytes: &[u8]) -> mp4ameta::ImgFmt {
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
                mp4ameta::ImgFmt::Jpeg
        } else if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                mp4ameta::ImgFmt::Png
        } else {
                mp4ameta::ImgFmt::Jpeg // 默认兜底
        }
}
