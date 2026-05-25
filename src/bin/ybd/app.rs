use reqwest::Client;
use yumi_bilibili_download::{error::Result, login::get_account, model::account::Account, url::UA};

use crate::{cache::load_user_from_file, config::Config, directories::APP_PATH};

#[derive(Debug)]
pub struct App {
    config: Config,
    account: Account,
    client: Client,
}

impl App {
    pub async fn new(is_login: bool) -> Result<Self> {
        let config = Config::new(Some(APP_PATH.config_path()))?;
        let client = Client::builder()
            .user_agent(UA)
            .cookie_store(true)
            .build()?;

        let account: Account = if is_login {
            load_user_from_file(APP_PATH.cache_path())?
        } else {
            get_account(&client).await?
        };

        Ok(App {
            config,
            account,
            client,
        })
    }
}
