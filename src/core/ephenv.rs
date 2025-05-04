use crate::core::app_config::get_app_config;
use std::collections::HashMap;

pub fn get_ephenvs() -> HashMap<String, String> {
    let app_config = get_app_config();
    if let Some(ephenvs) = app_config.get_ephenvs() {
        return ephenvs.clone();
    }
    HashMap::new()
}

pub fn set_ephenv(key: String, value: String) -> () {
    let mut app_config = get_app_config();
    app_config.set_ephenv(key, value);
}
