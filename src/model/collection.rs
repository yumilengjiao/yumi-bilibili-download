//! 收藏夹返回的数据格式

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionUrlResponse {
    pub code: i64,
    pub message: String,
    pub ttl: i64,
    pub data: CollectionData,
}

impl CollectionUrlResponse {
    pub fn get_data(&self) -> Result<&CollectionData> {
        self.valid()?;
        Ok(&self.data)
    }

    fn valid(&self) -> Result<()> {
        match self.code {
            0 => Ok(()),
            -101 => Err(Error::Normal("账号未登录".into())),
            -403 => Err(Error::Normal("权限不足，可能需要大会员".into())),
            -404 => Err(Error::Normal("视频不存在".into())),
            -412 => Err(Error::Normal("请求被拦截，请稍后再试".into())),
            -10403 => Err(Error::Normal("地区限制，无法观看".into())),
            _ => Err(Error::Normal(format!("未知错误码: {}", self.code))),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionData {
    pub info: Info,
    pub medias: Vec<Media>,
    #[serde(rename = "has_more")]
    pub has_more: bool,
    pub ttl: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub id: i64,
    pub fid: i64,
    pub mid: i64,
    pub attr: i64,
    pub title: String,
    pub cover: String,
    pub upper: Upper,
    #[serde(rename = "cover_type")]
    pub cover_type: i64,
    #[serde(rename = "cnt_info")]
    pub cnt_info: CntInfo,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub intro: Option<String>,
    pub ctime: i64,
    pub mtime: i64,
    pub state: i64,
    #[serde(rename = "fav_state")]
    pub fav_state: i64,
    #[serde(rename = "like_state")]
    pub like_state: i64,
    #[serde(rename = "media_count")]
    pub media_count: i64,
    #[serde(rename = "is_top")]
    pub is_top: bool,
    #[serde(rename = "is_kid_playlist")]
    pub is_kid_playlist: bool,
    #[serde(rename = "kid_playlist_desc")]
    pub kid_playlist_desc: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Upper {
    pub mid: i64,
    pub name: String,
    pub face: String,
    pub followed: bool,
    #[serde(rename = "vip_type")]
    pub vip_type: i64,
    #[serde(rename = "vip_statue")]
    pub vip_statue: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CntInfo {
    pub collect: i64,
    pub play: i64,
    #[serde(rename = "thumb_up")]
    pub thumb_up: i64,
    pub share: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub id: i64,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub title: String,
    pub cover: String,
    pub intro: String,
    pub page: i64,
    pub duration: i64,
    pub upper: Upper2,
    pub attr: i64,
    #[serde(rename = "cnt_info")]
    pub cnt_info: CntInfo2,
    pub link: String,
    pub ctime: i64,
    pub pubtime: i64,
    #[serde(rename = "fav_time")]
    pub fav_time: i64,
    #[serde(rename = "bv_id")]
    pub bv_id: String,
    pub bvid: String,
    pub season: Value,
    pub ogv: Value,
    pub ugc: Ugc,
    #[serde(rename = "media_list_link")]
    pub media_list_link: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Upper2 {
    pub mid: i64,
    pub name: String,
    pub face: String,
    #[serde(rename = "jump_link")]
    pub jump_link: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CntInfo2 {
    pub collect: i64,
    pub play: i64,
    pub danmaku: i64,
    pub vt: i64,
    #[serde(rename = "play_switch")]
    pub play_switch: i64,
    pub reply: i64,
    #[serde(rename = "view_text_1")]
    pub view_text_1: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ugc {
    #[serde(rename = "first_cid")]
    pub first_cid: i64,
}
