//! 请求参数模型

use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::error::{Error, Result};

const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

#[derive(Copy, Clone)]
pub enum Quality {
    K8 = 127,
    K4 = 120,
    FHD1080P60 = 116,
    FHD1080P = 80,
    HD720P = 64,
    SD480P = 32,
    LD360P = 16,
}

#[derive(Copy, Clone)]
pub enum Fnval {
    Mp4 = 0,
    Dash = 16,
    DashFull = 4048, // DASH + 4K + HDR + 杜比全开
}

/// 视频信息请求参数
///
/// 通过 [`VideoRequestParamBuilder`] 构造，签名相关字段自动计算。
///
/// * `bvid`: 视频唯一标识
/// * `cid`: 视频分P标识
/// * `qn`: 视频画质等级，见 [`Quality`]
/// * `fnval`: 视频流格式，见 [`Fnval`]
/// * `fourk`: 是否允许 4K，默认 true
/// * `fnver`: B站内部版本号，固定为 "0"，B站更新时可通过 Builder 覆盖
/// * `wts`: WBI 时间戳，构造时自动生成
/// * `img_key`: WBI 签名所需 img_key，从 B站接口获取
/// * `sub_key`: WBI 签名所需 sub_key，从 B站接口获取
/// * `w_rid`: WBI 签名结果，由 `wts`、`img_key`、`sub_key` 自动计算
pub struct VideoRequestParam {
    pub bvid: String,
    pub cid: String,
    pub qn: Quality,
    pub fnval: Fnval,
    pub fourk: bool,
    fnver: String,
    wts: String,
    img_key: String,
    sub_key: String,
    w_rid: String,
}

impl VideoRequestParam {
    pub fn to_query_string(&self) -> String {
        format!(
            "bvid={}&cid={}&qn={}&fnval={}&fnver={}&fourk={}&wts={}&w_rid={}",
            self.bvid,
            self.cid,
            self.qn as u32,
            self.fnval as u32,
            self.fnver,
            self.fourk as u8,
            self.wts,
            self.w_rid,
        )
    }
}

struct Wbi {
    img_key: String,
    sub_key: String,
}

impl Wbi {
    fn new(img_key: impl Into<String>, sub_key: impl Into<String>) -> Self {
        Self {
            img_key: img_key.into(),
            sub_key: sub_key.into(),
        }
    }

    fn mixin_key(&self) -> String {
        let orig = format!("{}{}", self.img_key, self.sub_key);
        let chars: Vec<char> = orig.chars().collect();
        MIXIN_KEY_ENC_TAB
            .iter()
            .filter_map(|&i| chars.get(i))
            .take(32)
            .collect()
    }

    /// 对参数进行 WBI 签名，签名依赖wts, 返回 (wts, w_rid)
    fn sign(&self, params: &mut BTreeMap<String, String>) -> (String, String) {
        let wts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let wts_str = wts.to_string();
        params.insert("wts".to_string(), wts_str.clone());

        let mixin_key = self.mixin_key();
        let query = params
            .iter()
            .map(|(k, v)| {
                let filtered: String = v.chars().filter(|c| !"!'()*".contains(*c)).collect();
                format!("{}={}", k, urlencoding::encode(&filtered))
            })
            .collect::<Vec<_>>()
            .join("&");

        let w_rid = format!("{:x}", md5::compute(format!("{}{}", query, mixin_key)));
        (wts_str, w_rid)
    }
}

pub struct VideoRequestParamBuilder {
    bvid: Option<String>,
    cid: Option<String>,
    qn: Quality,
    fnval: Fnval,
    fourk: bool,
    fnver: Option<String>,
}

impl Default for VideoRequestParamBuilder {
    fn default() -> Self {
        Self {
            bvid: None,
            cid: None,
            qn: Quality::FHD1080P, // 默认 1080P
            fnval: Fnval::Dash,    // 默认 DASH
            fourk: true,
            fnver: None, // 默认 "0"
        }
    }
}

impl VideoRequestParamBuilder {
    pub fn new(bvid: impl Into<String>, cid: impl Into<String>) -> Self {
        Self {
            bvid: Some(bvid.into()),
            cid: Some(cid.into()),
            ..Default::default()
        }
    }

    pub fn qn(mut self, qn: Quality) -> Self {
        self.qn = qn;
        self
    }

    pub fn fnval(mut self, fnval: Fnval) -> Self {
        self.fnval = fnval;
        self
    }

    pub fn allow_4k(mut self) -> Self {
        self.fourk = true;
        self
    }

    /// 一般不需要调用，B站改版本号时才用
    pub fn fnver(mut self, v: impl Into<String>) -> Self {
        self.fnver = Some(v.into());
        self
    }

    pub fn build(
        self,
        img_key: impl Into<String>,
        sub_key: impl Into<String>,
    ) -> Result<VideoRequestParam> {
        let bvid = self.bvid.ok_or(Error::Build("bvid".into()))?;
        let cid = self.cid.ok_or(Error::Build("cid".into()))?;

        let wbi = Wbi::new(img_key, sub_key);

        // 构造待签名参数
        let mut params = BTreeMap::new();
        params.insert("bvid".to_string(), bvid.clone());
        params.insert("cid".to_string(), cid.clone());
        params.insert("qn".to_string(), (self.qn as u32).to_string());
        params.insert("fnval".to_string(), (self.fnval as u32).to_string());
        params.insert("fourk".to_string(), (self.fourk as u8).to_string());

        let (wts, w_rid) = wbi.sign(&mut params);

        Ok(VideoRequestParam {
            bvid,
            cid,
            qn: self.qn,
            fnval: self.fnval,
            fourk: self.fourk,
            fnver: self.fnver.unwrap_or_else(|| "0".into()),
            img_key: wbi.img_key,
            sub_key: wbi.sub_key,
            wts,
            w_rid,
        })
    }
}
