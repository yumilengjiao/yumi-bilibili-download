use clap::ValueEnum;

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum VideoQuality {
        K8 = 127,
        K4 = 120,
        FHD1080P60 = 116,
        FHD1080P = 80,
        HD720P = 64,
        SD480P = 32,
        LD360P = 16,
}

#[derive(ValueEnum, Copy, Clone, Debug)]
pub enum VideoEncode {
        AVC,  // H.264，兼容性最好
        HEVC, // H.265，压缩率更高
        AV1,  // AV1，最新，压缩率最高
}

impl VideoEncode {
        pub fn as_str(&self) -> &'static str {
                // 过滤时通过start_with
                match self {
                        | Self::AVC => "avc",
                        | Self::HEVC => "hev",
                        | Self::AV1 => "av01",
                }
        }
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum AudioQuality {
        HiRes = 30251,  // Hi-Res 无损
        Dolby = 30250,  // 杜比全景声
        High = 30280,   // 192kbps
        Medium = 30232, // 132kbps
        Low = 30216,    // 64kbps
}
