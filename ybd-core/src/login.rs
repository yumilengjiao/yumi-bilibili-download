//! 用户登录与鉴权模块
//!
//! 提供二维码生成与渲染、登录状态轮询以及 WBI 签名密钥提取功能。

use std::time::SystemTime;


use qrcode::{QrCode, render::unicode};
use reqwest::Client;
use serde_json::Value;

use crate::{
        client::BiliClient,
        error::{Error, Result},
        model::account::Account,
        url::{LOGIN, SESSDATA, UA, VALIDATE_QRCODE, WBI},
};

/// 获取用户账户信息
///
/// * `client`: reqwest客户端,用于发送请求
pub async fn get_account() -> Result<Account> {
        let client = Client::builder().user_agent(UA).build()?;
        let qrcode_key = generate_qrcode_and_get_qrcode_key(&client).await?;
        let (user_id, sessdata, exp) = query_login_state(&qrcode_key, &client).await?;
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
                let resp = client
                        .get(VALIDATE_QRCODE)
                        .query(&[("qrcode_key", qrcode_key)])
                        .send()
                        .await?;

                let cookies: Vec<String> = resp
                        .headers()
                        .get_all("set-cookie")
                        .iter()
                        .filter_map(|v| v.to_str().ok().map(|s| s.to_string()))
                        .collect();

                let resp_value: Value = resp.json().await?;

                match resp_value["data"]["code"].as_i64().unwrap_or(-1) {
                        | 0 => {
                                let user_id = cookies
                                        .iter()
                                        .find(|c| c.starts_with("DedeUserID="))
                                        .and_then(|c| c.split(';').next())
                                        .and_then(|c| c.split('=').nth(1))
                                        .ok_or_else(|| Error::Normal("未找到 DedeUserID".into()))?
                                        .to_string();

                                let sessdata_cookie = cookies
                                        .iter()
                                        .find(|c| c.starts_with("SESSDATA="))
                                        .ok_or_else(|| Error::Normal("未找到 SESSDATA".into()))?;

                                let sessdata = sessdata_cookie
                                        .split(';')
                                        .next()
                                        .and_then(|s| s.split('=').nth(1))
                                        .ok_or_else(|| Error::Normal("SESSDATA 格式异常".into()))?
                                        .to_string();

                                let expire_time = sessdata_cookie
                                        .split("Expires=")
                                        .nth(1)
                                        .and_then(|s| s.split(';').next())
                                        .and_then(|s| httpdate::parse_http_date(s.trim()).ok())
                                        .ok_or_else(|| {
                                                Error::Normal("未能解析 SESSDATA 过期时间".into())
                                        })?;

                                return Ok((user_id, sessdata, expire_time));
                        },
                        | 86101 => print!("\r等待扫码..."),
                        | 86090 => print!("\r已扫码，请在手机上确认..."),
                        | 86038 => return Err(Error::Normal("二维码已过期，请重新运行".into())),
                        | code => println!("未知状态码: {}", code),
                }
        }
}

/// 获取wbi签名所需的img_key和sub_key密钥
///
/// * `client`: reqwest客户端
/// * `sessdata`: 会话令牌
///
/// # Retures
///
/// (img_key, sub_key)
pub async fn get_wbi_keys(client: &BiliClient) -> Result<(String, String)> {
        let resp: Value = client.get(WBI).send().await?.json().await?;

        let img_url = resp["data"]["wbi_img"]["img_url"]
                .as_str()
                .ok_or(Error::Normal("无法获取 img_url".into()))?;
        let sub_url = resp["data"]["wbi_img"]["sub_url"]
                .as_str()
                .ok_or(Error::Normal("无法获取 sub_url".into()))?;

        // 从 URL 中提取文件名（去掉路径和 .png 后缀）
        let img_key = img_url
                .split('/')
                .next_back()
                .unwrap_or("")
                .trim_end_matches(".png")
                .to_string();
        let sub_key = sub_url
                .split('/')
                .next_back()
                .unwrap_or("")
                .trim_end_matches(".png")
                .to_string();

        Ok((img_key, sub_key))
}
