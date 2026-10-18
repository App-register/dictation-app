use std::sync::{Arc, Mutex};
use dicto_core::{Config, Dictionary};

pub struct AppState {
    pub config: Arc<Mutex<Config>>,
    pub dictionary: Arc<Mutex<Dictionary>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(Config::load())),
            dictionary: Arc::new(Mutex::new(Dictionary::load())),
        }
    }
}