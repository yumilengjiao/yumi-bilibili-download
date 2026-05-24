// 登录
pub const LOGIN: &str = "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
pub const VALIDATE_QRCODE: &str =
    "https://passport.bilibili.com/x/passport-login/web/qrcode/poll?qrcode_key={qrcode_key}";

// WBI 签名密钥
pub const WBI: &str = "https://api.bilibili.com/x/web-interface/nav";

// 视频信息
pub const VEDIO_INFO: &str = "https://api.bilibili.com/x/web-interface/view?bvid={bvid}";

// 视频下载地址（需要 WBI 签名）
pub const VEDIO_DOWNLOAD_URL: &str = "https://api.bilibili.com/x/player/wbi/playurl?bvid={bvid}&cid={cid}&qn=80&fnval=0&fnver=0&fourk=1&w_rid={w_rid}&wts={wts}";

// UA
pub const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
