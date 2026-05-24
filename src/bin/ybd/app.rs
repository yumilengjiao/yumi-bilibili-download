use reqwest::Client;
use yumi_bilibili_download::{error::Result, login::get_account, model::account::Account, url::UA};

use crate::{config::Config, directories::APP_PATH};

#[derive(Debug)]
pub struct App {
    config: Config,
    account: Account,
    client: Client,
}

impl App {
    pub async fn new() -> Result<Self> {
        let config = Config::new(Some(APP_PATH.config_path()))?;
        let client = Client::builder()
            .user_agent(UA)
            .cookie_store(true)
            .build()?;
        let account = get_account(&client).await?;
        Ok(App {
            config,
            account,
            client,
        })
    }
}
