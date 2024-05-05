use crate::app_config::get_app_config;
use std::env;

pub fn get_env() -> Option<String> {
    if let Some(app_config) = get_app_config() {
        let current_dir = env::current_dir().unwrap().to_string_lossy().into_owned();

        if let Some(env) = app_config.envs.get(&current_dir) {
            return Some(env.clone());
        }
    }
    None
}
