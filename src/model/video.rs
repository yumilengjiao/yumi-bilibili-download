//! bilibili视频详细信息模型

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::client::BiliClient;
use crate::error::Error;
use crate::error::Result;
use crate::model::param::VideoRequestParamBuilder;
use crate::model::quality::AudioQuality;
use crate::model::quality::VideoEncode;
use crate::model::quality::VideoQuality;
use crate::url::VIDEO_DOWNLOAD_URL;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayUrlResponse {
    pub code: i64,
    data: VideoData,
    pub message: String,
    pub ttl: i64,
}

impl PlayUrlResponse {
    pub async fn new(bili_client: &BiliClient, bvid: &str) -> Result<Self> {
        // 这里构造的是视频请求参数,参数签名逻辑也封装在内
        let vrp = VideoRequestParamBuilder::new(bvid, bili_client)
            .await?
            .build(bili_client)
            .await?;
        let url = format!("{}?{}", VIDEO_DOWNLOAD_URL, vrp.to_query_string());
        // TODO: 老视频不兼容PlayUrlResponse这个结构
        Ok(bili_client.get(&url).send().await?.json().await?)
    }

    pub fn get_data(&self) -> Result<&VideoData> {
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
pub struct VideoData {
    #[serde(rename = "accept_description")]
    pub accept_description: Vec<String>,
    #[serde(rename = "accept_format")]
    pub accept_format: String,
    #[serde(rename = "accept_quality")]
    pub accept_quality: Vec<i64>,
    #[serde(rename = "auto_qn_resp")]
    pub auto_qn_resp: AutoQnResp,
    #[serde(rename = "cur_language")]
    pub cur_language: String,
    #[serde(rename = "cur_production_type")]
    pub cur_production_type: i64,
    // 老视频的信息中无此字段，直接抛出错误
    pub dash: Dash,
    pub format: String,
    pub from: String,
    #[serde(rename = "high_format")]
    pub high_format: Value,
    #[serde(rename = "last_play_cid")]
    pub last_play_cid: i64,
    #[serde(rename = "last_play_time")]
    pub last_play_time: i64,
    pub message: String,
    #[serde(rename = "play_conf")]
    pub play_conf: PlayConf,
    pub quality: i64,
    pub result: String,
    #[serde(rename = "seek_param")]
    pub seek_param: String,
    #[serde(rename = "seek_type")]
    pub seek_type: String,
    #[serde(rename = "support_formats")]
    pub support_formats: Vec<SupportFormat>,
    pub timelength: i64,
    #[serde(rename = "video_codecid")]
    pub video_codecid: i64,
    #[serde(rename = "view_info")]
    pub view_info: Value,
}

impl VideoData {
    pub fn best_video_quality_url(&self) -> Option<&str> {
        self.dash
            .video
            .iter()
            .filter(|v| self.accept_quality.contains(&v.id))
            .max_by_key(|v| v.id)
            .map(|v| v.base_url.as_str())
    }

    pub fn get_specified_video_url(
        &self,
        video_quality: Option<VideoQuality>,
        video_encode: Option<VideoEncode>,
    ) -> Option<&str> {
        if video_quality.is_none() && video_encode.is_none() {
            return None;
        }
        self.dash
            .video
            .iter()
            .filter(|v| self.accept_quality.contains(&v.id))
            .filter(|v| video_quality.is_none_or(|vq| v.id == vq as i64))
            .find(|v| video_encode.is_none_or(|ve| v.codecs.starts_with(ve.as_str())))
            .map(|v| v.base_url.as_str())
    }

    pub fn best_audio_url(&self) -> Option<&str> {
        if let Some(flac) = &self.dash.flac {
            return Some(&flac.audio.base_url);
        }
        self.dash
            .audio
            .iter()
            .max_by_key(|a| a.id)
            .map(|v| v.base_url.as_str())
    }

    pub fn get_specified_audio_url(&self, audio_quality: AudioQuality) -> Option<&str> {
        match audio_quality {
            AudioQuality::HiRes => self
                .dash
                .flac
                .as_ref()
                .map(|flac| flac.audio.base_url.as_str()),
            AudioQuality::Dolby => self
                .dash
                .dolby
                .as_ref()
                .and_then(|db| db.audio.as_ref())
                .and_then(|audio_vec| audio_vec.first())
                .map(|a| a.base_url.as_str()),
            _ => self
                .dash
                .audio
                .iter()
                .find(|a| a.id == audio_quality as i64)
                .map(|a| a.base_url.as_str()),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoQnResp {
    pub dyeid: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dash {
    pub audio: Vec<Audio>,
    pub dolby: Option<Dolby>,
    pub duration: i64,
    pub flac: Option<Flac>,
    pub min_buffer_time: f64,
    #[serde(rename = "min_buffer_time")]
    pub min_buffer_time2: f64,
    pub video: Vec<Video>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audio {
    #[serde(rename = "SegmentBase")]
    pub segment_base: SegmentBase,
    pub backup_url: Vec<String>,
    #[serde(rename = "backup_url")]
    pub backup_url2: Vec<String>,
    pub bandwidth: i64,
    pub base_url: String,
    #[serde(rename = "base_url")]
    pub base_url2: String,
    pub codecid: i64,
    pub codecs: String,
    pub frame_rate: String,
    #[serde(rename = "frame_rate")]
    pub frame_rate2: String,
    pub height: i64,
    pub id: i64,
    pub mime_type: String,
    #[serde(rename = "mime_type")]
    pub mime_type2: String,
    pub sar: String,
    #[serde(rename = "segment_base")]
    pub segment_base2: SegmentBase2,
    pub start_with_sap: i64,
    #[serde(rename = "start_with_sap")]
    pub start_with_sap2: i64,
    pub width: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentBase {
    #[serde(rename = "Initialization")]
    pub initialization: String,
    pub index_range: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentBase2 {
    #[serde(rename = "index_range")]
    pub index_range: String,
    pub initialization: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dolby {
    pub audio: Option<Vec<Audio2>>,
    #[serde(rename = "type")]
    pub type_field: i64,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audio2 {
    #[serde(rename = "SegmentBase")]
    pub segment_base: SegmentBase3,
    pub backup_url: Vec<String>,
    #[serde(rename = "backup_url")]
    pub backup_url2: Vec<String>,
    pub bandwidth: i64,
    pub base_url: String,
    #[serde(rename = "base_url")]
    pub base_url2: String,
    pub codecid: i64,
    pub codecs: String,
    pub frame_rate: String,
    #[serde(rename = "frame_rate")]
    pub frame_rate2: String,
    pub height: i64,
    pub id: i64,
    pub mime_type: String,
    #[serde(rename = "mime_type")]
    pub mime_type2: String,
    pub sar: String,
    #[serde(rename = "segment_base")]
    pub segment_base2: SegmentBase4,
    pub start_with_sap: i64,
    #[serde(rename = "start_with_sap")]
    pub start_with_sap2: i64,
    pub width: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    #[serde(rename = "SegmentBase")]
    pub segment_base: SegmentBase3,
    pub backup_url: Vec<String>,
    #[serde(rename = "backup_url")]
    pub backup_url2: Vec<String>,
    pub bandwidth: i64,
    pub base_url: String,
    #[serde(rename = "base_url")]
    pub base_url2: String,
    pub codecid: i64,
    pub codecs: String,
    pub frame_rate: String,
    #[serde(rename = "frame_rate")]
    pub frame_rate2: String,
    pub height: i64,
    pub id: i64,
    pub mime_type: String,
    #[serde(rename = "mime_type")]
    pub mime_type2: String,
    pub sar: String,
    #[serde(rename = "segment_base")]
    pub segment_base2: SegmentBase4,
    pub start_with_sap: i64,
    #[serde(rename = "start_with_sap")]
    pub start_with_sap2: i64,
    pub width: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentBase3 {
    #[serde(rename = "Initialization")]
    pub initialization: String,
    pub index_range: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentBase4 {
    #[serde(rename = "index_range")]
    pub index_range: String,
    pub initialization: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Flac {
    pub audio: Audio3,
    pub display: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Audio3 {
    #[serde(rename = "SegmentBase")]
    pub segment_base: SegmentBase5,
    pub backup_url: Vec<String>,
    #[serde(rename = "backup_url")]
    pub backup_url2: Vec<String>,
    pub bandwidth: i64,
    pub base_url: String,
    #[serde(rename = "base_url")]
    pub base_url2: String,
    pub codecid: i64,
    pub codecs: String,
    pub frame_rate: String,
    #[serde(rename = "frame_rate")]
    pub frame_rate2: String,
    pub height: i64,
    pub id: i64,
    pub mime_type: String,
    #[serde(rename = "mime_type")]
    pub mime_type2: String,
    pub sar: String,
    #[serde(rename = "segment_base")]
    pub segment_base2: SegmentBase6,
    pub start_with_sap: i64,
    #[serde(rename = "start_with_sap")]
    pub start_with_sap2: i64,
    pub width: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentBase5 {
    #[serde(rename = "Initialization")]
    pub initialization: String,
    pub index_range: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SegmentBase6 {
    #[serde(rename = "index_range")]
    pub index_range: String,
    pub initialization: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayConf {
    #[serde(rename = "is_new_description")]
    pub is_new_description: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportFormat {
    #[serde(rename = "can_watch_qn_reason")]
    pub can_watch_qn_reason: i64,
    pub codecs: Vec<String>,
    #[serde(rename = "display_desc")]
    pub display_desc: String,
    pub format: String,
    #[serde(rename = "limit_watch_reason")]
    pub limit_watch_reason: i64,
    #[serde(rename = "new_description")]
    pub new_description: String,
    pub quality: i64,
    pub report: Report,
    pub superscript: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {}
