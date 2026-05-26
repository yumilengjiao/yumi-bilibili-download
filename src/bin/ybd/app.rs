use yumi_bilibili_download::{error::Result, model::account::Account};

use crate::{cache::load_user_from_file, config::Config, directories::APP_PATH};

#[derive(Debug)]
pub struct App {
    pub config: Config,
    pub account: Option<Account>,
}

impl App {
    pub async fn new() -> Result<Self> {
        let config = Config::new(Some(APP_PATH.config_path()))?;
        let account = load_user_from_file(APP_PATH.cache_auth_path()).ok();

        Ok(App { config, account })
    }
}
