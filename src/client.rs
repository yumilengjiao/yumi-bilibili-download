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

    pub fn get(&self, url: &str) -> reqwest::RequestBuilder {
        self.client
            .get(url)
            .header("Cookie", format!("SESSDATA={}", self.sessdata))
            .header("Referer", "https://www.bilibili.com")
    }
}
