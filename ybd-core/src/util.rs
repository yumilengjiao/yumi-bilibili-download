//! 辅助工具模块
//!
//! 包含 BV 号/媒体列表 ID 提取、音频元数据内嵌（mp4ameta）及格式检测等实用工具。

use std::{path::Path, time::Duration};

use mp4ameta::{Chapter, Img, ImgFmt, Tag};
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
        let regex = Regex::new(r"ml(\d+)").expect("不正确的正则表达式");
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
        // 接口偶尔返回协议相对地址（//i0.hdslb.com/...），统一补成 https
        let mut url = if cover_url.starts_with("//") {
                format!("https://{}", cover_url.trim_start_matches('/'))
        } else {
                cover_url.to_string()
        };
        // 去掉 B 站 CDN 的处理后缀（`@672w_378h_1c.webp`），否则拿到的是 webp
        if let Some(index) = url.find('@') {
                url.truncate(index);
        }
        let response = client.get(&url).send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
}

/// 判断文件里是否已经有封面
pub fn check_cover_box(file_path: &Path) -> Result<bool> {
        let tag = Tag::read_from_path(file_path)?;
        Ok(tag.artwork().is_some())
}

/// 写入封面
///
/// * `file_path`: m4a/mp4 文件
/// * `image_data`: 图片原始字节（仅支持 jpeg/png/bmp）
pub fn add_cover_box(
        file_path: &Path,
        image_data: Vec<u8>,
) -> Result<()> {
        let fmt = detect_image_format(&image_data)?;
        let mut tag = Tag::read_from_path(file_path)?;
        let image = Img::new(fmt, image_data);
        tag.set_artwork(image);
        tag.write_to_path(file_path)?;
        Ok(())
}

fn detect_image_format(bytes: &[u8]) -> Result<ImgFmt> {
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
                Ok(ImgFmt::Jpeg)
        } else if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
                Ok(ImgFmt::Png)
        } else if bytes.starts_with(b"BM") {
                Ok(ImgFmt::Bmp)
        } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
                // mp4ameta 只认 jpeg/png/bmp，webp 直接写入会被播放器忽略
                Err(Error::Normal(
                        "封面是 webp 格式，mp4ameta 不支持，请先用 ?x-oss-process 转成 jpg".into(),
                ))
        } else {
                Err(Error::Normal("无法识别的封面图片格式".into()))
        }
}

/// 可写入 m4a/mp4 的元数据集合
///
/// 所有字段都是可选的，为 `None` 时保持文件原有内容不变。
#[derive(Default, Debug, Clone, PartialEq)]
pub struct MediaMeta {
        /// 标题（视频标题）
        pub title: Option<String>,
        /// 歌手（UP 主）
        pub artist: Option<String>,
        /// 专辑（收藏夹/合集/分区）
        pub album: Option<String>,
        pub album_artist: Option<String>,
        /// 备注（这里放视频原始链接）
        pub comment: Option<String>,
        /// 简介（视频描述）
        pub description: Option<String>,
        /// 流派（分区名）
        pub genre: Option<String>,
        pub year: Option<String>,
        /// (第几P, 共几P)
        pub track: Option<(u16, u16)>,
        /// 写入工具标识
        pub encoder: Option<String>,
        pub copyright: Option<String>,
        /// 歌词（可由字幕/同名 .lrc 转成 LRC 文本）
        pub lyrics: Option<String>,
        pub keywords: Vec<String>,
        /// 章节，仅对“多P/合集合并成单个文件”有意义
        pub chapters: Vec<(Duration, String)>,
}

impl MediaMeta {
        /// 把元数据合并进已有 tag（`None` 的字段不覆盖）
        pub fn apply(
                &self,
                tag: &mut Tag,
        ) {
                set_str(&self.title, |v| tag.set_title(v));
                set_str(&self.artist, |v| tag.set_artist(v));
                set_str(&self.album, |v| tag.set_album(v));
                set_str(&self.album_artist, |v| tag.set_album_artist(v));
                set_str(&self.comment, |v| tag.set_comment(v));
                set_str(&self.description, |v| tag.set_description(v));
                set_str(&self.genre, |v| tag.set_genre(v));
                set_str(&self.year, |v| tag.set_year(v));
                set_str(&self.encoder, |v| tag.set_encoder(v));
                set_str(&self.copyright, |v| tag.set_copyright(v));
                set_str(&self.lyrics, |v| tag.set_lyrics(v));
                if let Some((index, total)) = self.track {
                        tag.set_track(index, total);
                }
                if !self.keywords.is_empty() {
                        tag.set_keywords(self.keywords.clone());
                }
                if !self.chapters.is_empty() {
                        tag.chapters_mut().clear();
                        for (start, title) in &self.chapters {
                                tag.chapters_mut().push(Chapter::new(*start, title.clone()));
                        }
                }
        }
}

fn set_str(
        value: &Option<String>,
        setter: impl FnOnce(String),
) {
        if let Some(v) = value {
                setter(v.clone());
        }
}

/// 写入元数据（不含封面）
pub fn write_metadata(
        file_path: &Path,
        meta: &MediaMeta,
) -> Result<()> {
        let mut tag = Tag::read_from_path(file_path)?;
        meta.apply(&mut tag);
        tag.write_to_path(file_path)?;
        Ok(())
}

/// 一面写元数据、一面写封面的组合版本
///
/// * `cover`: 封面字节，`None` 表示跳过封面
pub fn write_metadata_with_cover(
        file_path: &Path,
        meta: &MediaMeta,
        cover: Option<Vec<u8>>,
) -> Result<()> {
        let mut tag = Tag::read_from_path(file_path)?;
        meta.apply(&mut tag);
        if let Some(bytes) = cover {
                let fmt = detect_image_format(&bytes)?;
                tag.set_artwork(Img::new(fmt, bytes));
        }
        tag.write_to_path(file_path)?;
        Ok(())
}

/// 判断文件元数据是否已经与目标一致（一致则无需重写）
///
/// 只对比“关键字段”，避免简介这类长文本变动导致反复写盘。
pub fn check_metadata(
        file_path: &Path,
        meta: &MediaMeta,
) -> Result<bool> {
        let tag = Tag::read_from_path(file_path)?;
        let same = |actual: Option<&str>, expected: &Option<String>| match expected {
                Some(e) => actual == Some(e.as_str()),
                None => true,
        };
        let ok = same(tag.title(), &meta.title)
                && same(tag.artist(), &meta.artist)
                && same(tag.album(), &meta.album)
                && same(tag.comment(), &meta.comment);
        Ok(ok)
}

/// 读取同名 `.lrc` 字幕/歌词，作为 `lyrics` 字段的内容
///
/// * `media_path`: 音频/视频文件路径，函数会自动把扩展名换成 `.lrc`
pub fn read_lyrics_sidecar(media_path: &Path) -> Option<String> {
        let lrc = media_path.with_extension("lrc");
        let text = std::fs::read_to_string(lrc).ok()?;
        if text.trim().is_empty() {
                None
        } else {
                Some(text)
        }
}
