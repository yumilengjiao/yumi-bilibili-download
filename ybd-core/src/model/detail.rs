//! bilibili 视频详细信息模型（对应 `x/web-interface/view` 接口）
//!
//! 与 [`crate::model::video::PlayUrlResponse`] 不同，这个接口提供的是
//! 标题、UP 主、发布时间、简介、分P 列表等“可写入文件元数据”的信息。

use serde::Deserialize;
use serde::Serialize;

use crate::util::MediaMeta;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfoResponse {
        pub code: i64,
        #[serde(default)]
        pub message: String,
        #[serde(default)]
        pub data: Option<VideoDetail>,
}

impl VideoInfoResponse {
        /// 提取数据部分，顺手把常见错误码翻译成可读信息
        pub fn into_detail(self) -> crate::error::Result<VideoDetail> {
                if self.code != 0 {
                        return Err(crate::error::Error::Normal(format!(
                                "获取视频信息失败: {} (code: {})",
                                self.message, self.code
                        )));
                }
                self.data.ok_or_else(|| {
                        crate::error::Error::Normal("视频信息返回体缺少 data 字段".into())
                })
        }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDetail {
        pub bvid: String,
        #[serde(default)]
        pub aid: i64,
        pub title: String,
        #[serde(default)]
        pub pic: String,
        /// 视频简介
        #[serde(default)]
        pub desc: String,
        /// 分区名称，例如“音乐 / 演奏”
        #[serde(default)]
        pub tname: String,
        #[serde(default)]
        pub tid: i64,
        /// 发布时间戳（秒）
        #[serde(default)]
        pub pubdate: i64,
        /// 创建时间戳（秒）
        #[serde(default)]
        pub ctime: i64,
        /// 总时长（秒）
        #[serde(default)]
        pub duration: i64,
        /// 分P 数量
        #[serde(default)]
        pub videos: i64,
        /// 1: 原创, 2: 转载
        #[serde(default)]
        pub copyright: i64,
        #[serde(default)]
        pub cid: i64,
        #[serde(default)]
        pub owner: Owner,
        #[serde(default)]
        pub stat: Stat,
        #[serde(default)]
        pub pages: Vec<VideoPage>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
        #[serde(default)]
        pub mid: i64,
        #[serde(default)]
        pub name: String,
        #[serde(default)]
        pub face: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stat {
        #[serde(default)]
        pub view: i64,
        #[serde(default)]
        pub danmaku: i64,
        #[serde(default)]
        pub reply: i64,
        #[serde(default)]
        pub favorite: i64,
        #[serde(default)]
        pub coin: i64,
        #[serde(default)]
        pub share: i64,
        #[serde(default)]
        pub like: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoPage {
        #[serde(default)]
        pub cid: i64,
        /// 第几 P，从 1 开始
        #[serde(default)]
        pub page: i64,
        /// 分P 标题
        #[serde(default)]
        pub part: String,
        #[serde(default)]
        pub duration: i64,
}

impl VideoDetail {
        /// 视频原始地址（带分P 参数）
        pub fn url(
                &self,
                cid: Option<i64>,
        ) -> String {
                match cid {
                        Some(cid) => format!(
                                "https://www.bilibili.com/video/{}/?p={}",
                                self.bvid,
                                self.page_index(cid).unwrap_or(1)
                        ),
                        None => format!("https://www.bilibili.com/video/{}", self.bvid),
                }
        }

        /// 根据 cid 找到对应的分P
        pub fn page_of(
                &self,
                cid: i64,
        ) -> Option<&VideoPage> {
                self.pages.iter().find(|p| p.cid == cid)
        }

        /// cid 对应的分P 序号（从 1 开始）
        pub fn page_index(
                &self,
                cid: i64,
        ) -> Option<u16> {
                self.page_of(cid)
                        .map(|p| p.page as u16)
                        .or(if self.pages.is_empty() { Some(1) } else { None })
        }

        /// 发布时间所在年份
        pub fn year(&self) -> Option<String> {
                let ts = if self.pubdate > 0 {
                        self.pubdate
                } else {
                        self.ctime
                };
                if ts <= 0 {
                        return None;
                }
                // 不引入时间库，手工把时间戳换算成年月日（UTC 近似即可满足元数据需求）
                let days = ts / 86_400;
                let (y, m, d) = civil_from_days(days);
                let _ = (m, d);
                Some(y.to_string())
        }

        /// 组装可写入 m4a/mp4 的元数据
        ///
        /// * `album`: 合集/收藏夹名称，缺省时用分区名
        /// * `cid`: 要下载的分P 标识，用于写入 track 序号
        pub fn media_meta(
                &self,
                album: Option<&str>,
                cid: Option<i64>,
        ) -> MediaMeta {
                let total = self.pages.len().max(if self.videos > 0 { 1 } else { 0 }) as u16;
                let track = cid
                        .and_then(|cid| self.page_index(cid))
                        .map(|index| (index, total.max(index)));

                let mut meta = MediaMeta::default();
                meta.title = Some(self.title.clone());
                if !self.owner.name.is_empty() {
                        meta.artist = Some(self.owner.name.clone());
                        meta.album_artist = Some(self.owner.name.clone());
                        meta.copyright = Some(format!("© {}", self.owner.name));
                }
                meta.album = album.map(|s| s.to_string()).or(if self.tname.is_empty() {
                        None
                } else {
                        Some(self.tname.clone())
                });
                meta.genre = if self.tname.is_empty() {
                        None
                } else {
                        Some(self.tname.clone())
                };
                meta.description = if self.desc.is_empty() {
                        None
                } else {
                        Some(self.desc.clone())
                };
                meta.comment = Some(self.url(cid));
                meta.year = self.year();
                meta.track = track;
                meta.encoder = Some(format!(
                        "{} {}",
                        env!("CARGO_PKG_NAME"),
                        env!("CARGO_PKG_VERSION")
                ));
                meta
        }
}

/// 把“1970-01-01 起的天数”换算成公历年月日
///
/// 采用 Howard Hinnant 的 civil_from_days 算法，避免为了一个年份引入时间库。
fn civil_from_days(days: i64) -> (i64, u32, u32) {
        let z = days + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let doe = (z - era * 146_097) as u64;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        (if m <= 2 { y + 1 } else { y }, m, d)
}

#[test]
fn year_of_timestamp() {
        let detail = VideoDetail {
                pubdate: 1_700_000_000,
                ..Default::default()
        };
        assert_eq!(detail.year().as_deref(), Some("2023"));
}
