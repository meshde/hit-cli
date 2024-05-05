use crate::app_config::{get_app_config, AppConfig};
use std::env;

pub fn get_env_key() -> String {
    env::current_dir().unwrap().to_string_lossy().into_owned()
}

pub fn get_env() -> Option<String> {
    if let Some(app_config) = get_app_config() {
        let current_dir = get_env_key();

        if let Some(env) = app_config.envs.get(&current_dir) {
            return Some(env.clone());
        }
    }
    None
}

pub fn set_env(env: String) -> () {
    let mut app_config = match get_app_config() {
        Some(config) => config.clone(),
        None => AppConfig::new(),
    };

    app_config.envs.insert(get_env_key(), env);
    app_config.save();
}
