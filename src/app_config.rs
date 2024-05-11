use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Clone)]
pub struct AppConfig {
    #[serde(default)]
    pub envs: HashMap<String, String>,
    #[serde(default)]
    pub ephenvs: HashMap<String, HashMap<String, String>>,
}

impl AppConfig {
    pub fn new() -> AppConfig {
        AppConfig {
            envs: HashMap::new(),
            ephenvs: HashMap::new(),
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
}

fn get_app_config_dir() -> PathBuf {
    ProjectDirs::from("tech", "meshde", "hit-cli")
        .unwrap()
        .config_dir()
        .to_path_buf()
}

pub fn get_app_config_file_path() -> String {
    get_app_config_dir()
        .join("config.json")
        .to_string_lossy()
        .into_owned()
}
pub fn get_app_config() -> Option<AppConfig> {
    let config_file_path = get_app_config_file_path();
    if let Ok(file) = File::open(config_file_path) {
        let reader = BufReader::new(file);

        let app_config = serde_json::from_reader(reader).unwrap();
        return app_config;
    }
    None
}
