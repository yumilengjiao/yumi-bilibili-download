use yumi_bilibili_download::{error::Result, login::Account};

use crate::{config::Config, directories::APP_PATH};

#[derive(Debug)]
pub struct App {
    config: Config,
    account: Account,
}

impl App {
    pub async fn new() -> Result<Self> {
        let config = Config::new(Some(APP_PATH.config_path()))?;
        let account = Account::new().await?;
        Ok(App { config, account })
    }
}
