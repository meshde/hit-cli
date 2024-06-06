use crate::app_config::{get_app_config, AppConfig};
use std::collections::HashMap;

pub fn get_ephenvs() -> HashMap<String, String> {
    if let Some(app_config) = get_app_config() {
        if let Some(ephenvs) = app_config.get_ephenvs() {
            return ephenvs.clone();
        }
    }
    HashMap::new()
}

pub fn set_ephenv(key: String, value: String) -> () {
    let mut app_config = match get_app_config() {
        Some(config) => config.clone(),
        None => AppConfig::new(),
    };

    app_config.set_ephenv(key, value);
}
