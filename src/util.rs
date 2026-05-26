use regex::Regex;

use crate::error::{Error, Result};

/// 从链接提取bvid
///
/// * `bv_id`: 视频url
pub fn extract_bv_id(input: &str) -> Result<String> {
    let regex = Regex::new(r"BV[a-z0-9A-Z]+").expect("不正确的正则表达式");
    regex
        .find(input)
        .map(|m| m.as_str().to_string())
        .ok_or(Error::Normal("无法从输入中提取bv号".into()))
}
// eg. https://www.bilibili.com/list/ml2408095182?spm_id_from=333.1007.0.0&oid=115724639078994&bvid=BV1S4qWBkEGk
pub fn extract_media_id(input: &str) -> Result<String> {
    let regex = Regex::new(r"/ml(\d+)\?").expect("不正确的正则表达式");
    regex
        .captures(input)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or(Error::Normal("无法从输入中提取mlid号".into()))
}
