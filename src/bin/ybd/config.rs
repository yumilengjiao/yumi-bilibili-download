use std::thread;

pub struct Config {
    concurrencies: usize,
}

impl Config {
    pub fn new() -> Self {
        Config {
            concurrencies: thread::available_parallelism().unwrap().get(),
        }
    }
}
