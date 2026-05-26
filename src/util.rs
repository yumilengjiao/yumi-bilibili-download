use regex::Regex;

use crate::error::{Error, Result};

/// 提取bvid
///
/// * `bv_id`: 视频url
fn extract_bv_id(input: &str) -> Result<String> {
    let regex = Regex::new(r"BV[a-z0-9A-Z]+").expect("不正确的正则表达式");
    regex
        .find(input)
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| Error::Normal("无法从输入中提取bv号".into()))
}
