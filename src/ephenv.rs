use crate::app_config::{get_app_config, AppConfig};
use std::collections::HashMap;
use std::env;

pub fn get_env_key() -> String {
    env::current_dir().unwrap().to_string_lossy().into_owned()
}

pub fn get_ephenvs() -> HashMap<String, String> {
    if let Some(app_config) = get_app_config() {
        let current_dir = get_env_key();

        if let Some(ephenvs) = app_config.ephenvs.get(&current_dir) {
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

    let current_dir_key = get_env_key();

    app_config
        .ephenvs
        .entry(current_dir_key)
        .and_modify(|data| {
            data.insert(key.clone(), value.clone());
        })
        .or_insert_with(|| {
            let mut new_data = HashMap::new();
            new_data.insert(key, value);
            new_data
        });

    app_config.save();
}
