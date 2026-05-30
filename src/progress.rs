use std::{collections::HashMap, time::Duration};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub struct DownloadProgress {
    pub id: String,
    pub pb: ProgressBar,
}

impl DownloadProgress {
    pub fn new(id: String, title: String, total_size: u64) -> Self {
        let pb = ProgressBar::new(total_size);
        let short_title = {
            let mut result = String::new();
            let mut width = 0;
            for c in title.chars() {
                let cw = if (c as u32) > 0x7F { 2 } else { 1 };
                if width + cw > 12 {
                    result.push_str("...");
                    break;
                }
                result.push(c);
                width += cw;
            }
            result
        };
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{bar:25.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}",
            )
            .unwrap()
            .progress_chars("█▓▒░  "),
        );
        pb.set_message(short_title);
        Self { id, pb }
    }

    pub fn update(&mut self, inc: u64) {
        if self.pb.position() + inc >= self.pb.length().unwrap_or(0) {
            self.pb.inc(inc);
            self.pb.finish();
        }
        self.pb.inc(inc);
    }
}

pub struct DownloadMutiProgess {
    progress_bars: HashMap<String, ProgressBar>,
    last_progress_bar: ProgressBar,
    multi_progress: MultiProgress,
}

impl DownloadMutiProgess {
    /// 这里total_number限定总进度条的上限，而不是取progres_bars的长度,
    /// 处于美观需要，不会在add方法里面动态增长进度条上限
    ///
    /// * `progres_bars`: 初始进度条，一般是空向量
    /// * `total_number`: 进度条上限
    pub fn new(progres_bars: Vec<DownloadProgress>, total_number: u64) -> Self {
        let mut hm = HashMap::new();
        let mp = MultiProgress::new();

        // 总进度条先加，保证在最底部
        let last_progress_bar = mp.add(ProgressBar::new(total_number));
        last_progress_bar.set_style(
            ProgressStyle::with_template("总进度: [{bar:40.cyan/blue}] {pos}/{len} 个")
                .unwrap()
                .progress_chars("=>-"),
        );
        last_progress_bar.enable_steady_tick(Duration::from_millis(80));

        // 初始传入的进度条都 insert_before
        for v in progres_bars {
            let pb = mp.insert_before(&last_progress_bar, v.pb);
            hm.insert(v.id, pb);
        }

        Self {
            progress_bars: hm,
            last_progress_bar,
            multi_progress: mp,
        }
    }

    pub fn add(&mut self, progress: DownloadProgress) {
        let pb = self
            .multi_progress
            .insert_before(&self.last_progress_bar, progress.pb);
        self.progress_bars.insert(progress.id, pb);
    }

    pub fn update(&mut self, id: &str, inc_len: u64) {
        let v = self.progress_bars.get(id);
        if let Some(progress) = v {
            if progress.position() + inc_len >= progress.length().unwrap_or(0) {
                progress.inc(inc_len);
                progress.finish_and_clear();
                self.last_progress_bar.inc(1);
                return;
            }
            progress.inc(inc_len);
        }
    }

    pub fn inc_total(&mut self) {
        self.last_progress_bar.inc(1);
    }

    pub fn finish(&mut self) {
        self.last_progress_bar.finish();
    }
}
