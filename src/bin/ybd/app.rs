use crate::config::Config;

pub struct App {
    config: Config,
}

impl App {
    fn new() -> Self {
        App {
            config: Config::new(),
        }
    }
}
