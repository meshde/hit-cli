use directories::ProjectDirs;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(serde::Deserialize)]
pub struct AppConfig {
    pub envs: HashMap<String, String>,
}

pub fn get_app_config_file_path() -> String {
    Path::new(
        ProjectDirs::from("tech", "meshde", "hit-cli")
            .unwrap()
            .config_dir(),
    )
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
