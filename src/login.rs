use std::{
    num::ParseIntError,
    time::{Duration, SystemTime},
};

use qrcode::{QrCode, render::unicode};
use reqwest::Client;
use serde_json::Value;

use crate::{
    error::{Error, Result},
    model::account::Account,
    url::{LOGIN, UA, VALIDATE_QRCODE, WBI},
};

const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

pub async fn get_account(client: &Client) -> Result<Account> {
    let qrcode_key = generate_qrcode_and_get_qrcode_key(client).await?;
    let (user_id, sessdata, exp) = query_login_state(&qrcode_key, client).await?;
    Ok(Account::new(user_id, exp, sessdata))
}

/// 生成qrcode并获取qrcode_key用于轮训
///
/// * `client`: reqwest客户端
async fn generate_qrcode_and_get_qrcode_key(client: &Client) -> Result<String> {
    let resp: Value = client.get(LOGIN).send().await?.json().await?;
    let url = resp["data"]["url"]
        .as_str()
        .ok_or(Error::Normal("无法获取qrcode的URL字段".into()))?;
    let qrcode_key = resp["data"]["qrcode_key"]
        .as_str()
        .ok_or(Error::Normal("无法获取qrcode_key".into()))?
        .to_string();

    let code = QrCode::new(url.as_bytes()).map_err(|e| Error::Normal(e.to_string()))?;
    let image = code.render::<unicode::Dense1x2>().quiet_zone(false).build();
    println!("{}", image);
    println!("请用 手机相册 扫码登录...\n");
    Ok(qrcode_key)
}

/// 轮询查询QRCODE，即用户是否已经扫码登录
/// 获取用户的SESSDATA
///
/// # Arguments
/// * `qrcode_key` - qrcode key
/// * `client` - reqwest客户端
///
/// # Returns
/// * `Ok(String,String,SystemTime)` - 用户的ID,SESSDATA,时间戳
/// * `Err(Error)` - 二维码过期或网络错误
async fn query_login_state(
    qrcode_key: &str,
    client: &Client,
) -> Result<(String, String, SystemTime)> {
    // 轮询登录状态
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let resp: Value = client
            .get(VALIDATE_QRCODE)
            .query(&[("qrcode_key", &qrcode_key)])
            .send()
            .await?
            .json()
            .await?;

        match resp["data"]["code"].as_i64().unwrap_or(-1) {
            0 => {
                let url = resp["data"]["url"].as_str().unwrap_or("");
                let user_id = url
                    .split("DedeUserID=")
                    .nth(1)
                    .unwrap_or("")
                    .split("&")
                    .next()
                    .unwrap_or("")
                    .to_string();

                let exp = url
                    .split("Expires")
                    .nth(1)
                    .unwrap_or("")
                    .split("&")
                    .next()
                    .unwrap_or("")
                    .to_string();

                let sessdata = url
                    .split("SESSDATA=")
                    .nth(1)
                    .unwrap_or("")
                    .split('&')
                    .next()
                    .unwrap_or("")
                    .to_string();
                println!("✓ 登录成功！\n");
                let ts: u64 = exp
                    .parse::<u64>()
                    .map_err(|e| Error::Normal(e.to_string()))?;
                let exp = SystemTime::UNIX_EPOCH + Duration::from_secs(ts);
                return Ok((user_id, sessdata, exp));
            }
            86101 => print!("\r等待扫码..."),
            86090 => print!("\r已扫码，请在手机上确认..."),
            86038 => return Err(Error::Normal("二维码已过期，请重新运行".into())),
            code => println!("未知状态码: {}", code),
        }
    }
}

/// 获取wbi签名密钥
///
/// * `client`: reqwest客户端
/// * `sessdata`: 会话令牌
async fn get_wbi_keys(client: &Client, sessdata: &str) -> Result<(String, String)> {
    let resp: Value = client
        .get(WBI)
        .header("Cookie", format!("SESSDATA={}", sessdata))
        .send()
        .await?
        .json()
        .await?;

    let img_url = resp["data"]["wbi_img"]["img_url"]
        .as_str()
        .ok_or(Error::Normal("无法获取 img_url".into()))?;
    let sub_url = resp["data"]["wbi_img"]["sub_url"]
        .as_str()
        .ok_or(Error::Normal("无法获取 sub_url".into()))?;

    // 从 URL 中提取文件名（去掉路径和 .png 后缀）
    let img_key = img_url
        .split('/')
        .last()
        .unwrap_or("")
        .trim_end_matches(".png")
        .to_string();
    let sub_key = sub_url
        .split('/')
        .last()
        .unwrap_or("")
        .trim_end_matches(".png")
        .to_string();

    Ok((img_key, sub_key))
}
