use crate::http::Response;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::env;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Clone)]
pub struct AppConfig {
    #[serde(default)]
    envs: HashMap<String, String>,
    #[serde(default)]
    ephenvs: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    prev_request: HashMap<String, Response>,
}

impl AppConfig {
    pub fn new() -> AppConfig {
        AppConfig {
            envs: HashMap::new(),
            ephenvs: HashMap::new(),
            prev_request: HashMap::new(),
        }
    }

    pub fn save(&self) -> () {
        let app_config_dir = get_app_config_dir();
        let app_config_file_path = get_app_config_file_path();

        let json_string = serde_json::to_string_pretty(&self).unwrap();

        create_dir_all(app_config_dir).expect("could not create app config dir");
        let mut file =
            File::create(app_config_file_path).expect("could not create app config file");

        file.write_all(json_string.as_bytes())
            .expect("could not save app config");
    }

    pub fn get_current_env(&self) -> Option<&String> {
        self.envs.get(&get_config_key())
    }

    pub fn set_current_env(&mut self, env: String) {
        self.envs.insert(get_config_key(), env);
        self.save();
    }

    pub fn get_ephenvs(&self) -> Option<&HashMap<String, String>> {
        self.ephenvs.get(&get_config_key())
    }

    pub fn set_ephenv(&mut self, key: String, value: String) {
        self.ephenvs
            .entry(get_config_key())
            .and_modify(|data| {
                data.insert(key.clone(), value.clone());
            })
            .or_insert_with(|| {
                let mut new_data = HashMap::new();
                new_data.insert(key, value);
                new_data
            });
        self.save();
    }

    pub fn set_prev_request(&mut self, prev_request: Response) {
        self.prev_request.insert(get_config_key(), prev_request);
        self.save();
    }

    pub fn get_prev_request(&self) -> Option<&Response> {
        self.prev_request.get(&get_config_key())
    }
}

fn get_config_key() -> String {
    env::current_dir().unwrap().to_string_lossy().into_owned()
}

fn get_app_config_dir() -> PathBuf {
    ProjectDirs::from("tech", "meshde", "hit-cli")
        .unwrap()
        .config_dir()
        .to_path_buf()
}

pub fn get_app_config_file_path() -> String {
    #[cfg(test)]
    {
        if let Some(test_path) = crate::fixtures::get_test_config_path() {
            return test_path;
        }
    }
    
    get_app_config_dir()
        .join("config.json")
        .to_string_lossy()
        .into_owned()
}

pub fn get_app_config() -> AppConfig {
    let config_file_path = get_app_config_file_path();
    if let Ok(file) = File::open(config_file_path) {
        let reader = BufReader::new(file);

        let app_config = serde_json::from_reader(reader).unwrap();
        return app_config;
    }
    AppConfig::new()
}
