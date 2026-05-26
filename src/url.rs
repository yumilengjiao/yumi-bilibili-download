// 登录
pub const LOGIN: &str = "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
pub const VALIDATE_QRCODE: &str = "https://passport.bilibili.com/x/passport-login/web/qrcode/poll";

// WBI 签名密钥
pub const WBI: &str = "https://api.bilibili.com/x/web-interface/nav";

// 视频基础信息
pub const VIDEO_INFO: &str = "https://api.bilibili.com/x/web-interface/view";

// 收藏夹视频信息
pub const MEDIO_LIST: &str = "https://api.bilibili.com/x/v3/fav/resource/list";

// 视频下载地址（需要 WBI 签名）
pub const VIDEO_DOWNLOAD_URL: &str = "https://api.bilibili.com/x/player/wbi/playurl";

// UA
pub const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
