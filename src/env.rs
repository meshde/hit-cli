use crate::app_config::get_app_config;
use crate::config::Config;

pub fn get_env() -> Option<String> {
    let app_config = get_app_config();
    if let Some(env) = app_config.get_current_env() {
        return Some(env.clone());
    }
    None
}

pub fn set_env(env: String) -> () {
    let mut app_config = get_app_config();
    app_config.set_current_env(env);
}

pub fn list_envs() -> Vec<String> {
    Config::new().envs.keys().map(|k| k.clone()).collect()
}
