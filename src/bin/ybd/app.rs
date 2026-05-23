use yumi_bilibili_download::error::Result;

use crate::{config::Config, directories::APP_PATH};

#[derive(Debug)]
pub struct App {
    config: Config,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::new(Some(APP_PATH.config_path()))?;
        Ok(App { config })
    }
}
