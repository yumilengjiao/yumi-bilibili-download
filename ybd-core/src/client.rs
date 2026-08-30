//! HTTP 客户端封装模块
//!
//! 包装带有 Bilibili 用户鉴权凭证（Cookie/SESSDATA 与 Referer）的请求客户端。

use reqwest::Client;


use crate::{error::Result, model::account::Account, url::UA};

#[derive(Debug)]
pub struct BiliClient {
        client: Client,
        sessdata: String,
}

impl BiliClient {
        pub fn new(account: &Account) -> Result<Self> {
                let client = Client::builder().user_agent(UA).build()?;
                Ok(Self {
                        client,
                        sessdata: account.get_sessdata().into(),
                })
        }

        pub fn get(
                &self,
                url: &str,
        ) -> reqwest::RequestBuilder {
                self.client
                        .get(url)
                        .header("Cookie", format!("SESSDATA={}", self.sessdata))
                        .header("Referer", "https://www.bilibili.com")
        }

        pub fn downgrade(&self) -> &Client {
                &self.client
        }
}
