use std::{path::Path, sync::Arc};

use crate::model::quality::{AudioQuality, VideoEncode, VideoQuality};

/// 用于构造可选的下载参数
///
/// * `audio_path`: 输出的音频文件路径
/// * `video_path`: 输出视频(无音频)文件路径
/// * `output`: 指定最后合并视频音频后输出的视频的路径,其他下载方法(单独下视频和音频)不生效
/// * `video_quality`: 视频质量
/// * `video_encode`: 视频编码格式
/// * `audio_quality`: 音频质量
/// * `ffmpeg_path`: ffmpeg路径,不设置时从系统变量获取
pub struct DownloadOption<'a> {
        pub audio_path: Option<&'a Path>,
        pub video_path: Option<&'a Path>,
        pub output: Option<&'a Path>,
        pub video_quality: Option<VideoQuality>,
        pub video_encode: Option<VideoEncode>,
        pub audio_quality: Option<AudioQuality>,
        pub ffmpeg_path: Option<&'a Path>,
        pub on_video_progress: Option<ProgressCallback>,
        pub on_audio_progress: Option<ProgressCallback>,
}

impl<'a> DownloadOption<'a> {
        pub fn builder() -> DownloadOptionBuilder<'a> {
                DownloadOptionBuilder::new()
        }
}

pub type ProgressCallback = Arc<dyn Fn(u64, Option<u64>) + Send + Sync>;

pub struct DownloadOptionBuilder<'a> {
        audio_path: Option<&'a Path>,
        video_path: Option<&'a Path>,
        output: Option<&'a Path>,
        video_quality: Option<VideoQuality>,
        video_encode: Option<VideoEncode>,
        audio_quality: Option<AudioQuality>,
        ffmpeg_path: Option<&'a Path>,
        on_video_progress: Option<ProgressCallback>,
        on_audio_progress: Option<ProgressCallback>,
}

impl<'a> DownloadOptionBuilder<'a> {
        pub fn new() -> Self {
                Self {
                        audio_path: None,
                        video_path: None,
                        output: None,
                        video_quality: None,
                        video_encode: None,
                        audio_quality: None,
                        ffmpeg_path: None,
                        on_video_progress: None,
                        on_audio_progress: None,
                }
        }
        pub fn audio_path(
                mut self,
                audio_path: &'a Path,
        ) -> Self {
                self.audio_path = Some(audio_path);
                self
        }
        pub fn video_path(
                mut self,
                video_path: &'a Path,
        ) -> Self {
                self.video_path = Some(video_path);
                self
        }
        pub fn output(
                mut self,
                output: &'a Path,
        ) -> Self {
                self.output = Some(output);
                self
        }
        pub fn video_quality(
                mut self,
                video_quality: VideoQuality,
        ) -> Self {
                self.video_quality = Some(video_quality);
                self
        }
        pub fn video_encode(
                mut self,
                video_encode: VideoEncode,
        ) -> Self {
                self.video_encode = Some(video_encode);
                self
        }
        pub fn audio_quality(
                mut self,
                audio_quality: AudioQuality,
        ) -> Self {
                self.audio_quality = Some(audio_quality);
                self
        }
        pub fn ffmpeg_path(
                mut self,
                ffmpeg_path: &'a Path,
        ) -> Self {
                self.ffmpeg_path = Some(ffmpeg_path);
                self
        }
        pub fn on_video_progress(
                mut self,
                on_progress: ProgressCallback,
        ) -> Self {
                self.on_video_progress = Some(on_progress);
                self
        }
        pub fn on_audio_progress(
                mut self,
                on_progress: ProgressCallback,
        ) -> Self {
                self.on_audio_progress = Some(on_progress);
                self
        }
        pub fn build(self) -> DownloadOption<'a> {
                DownloadOption {
                        audio_path: self.audio_path,
                        video_path: self.video_path,
                        output: self.output,
                        video_quality: self.video_quality,
                        video_encode: self.video_encode,
                        audio_quality: self.audio_quality,
                        ffmpeg_path: self.ffmpeg_path,
                        on_video_progress: self.on_video_progress,
                        on_audio_progress: self.on_audio_progress,
                }
        }
}

impl<'a> Default for DownloadOptionBuilder<'a> {
        fn default() -> Self {
                Self::new()
        }
}
