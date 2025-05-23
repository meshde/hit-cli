use crate::constants::CONFIG_DIR;
use crate::core::command::Command;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Write};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub envs: HashMap<String, HashMap<String, String>>,
    pub commands: HashMap<String, Box<CommandType>>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum CommandType {
    Command(Command),
    NestedCommand(HashMap<String, Box<CommandType>>),
}

fn get_config_file_path() -> PathBuf {
    PathBuf::from(CONFIG_DIR).join("config.json")
}

impl Config {
    pub fn new() -> Config {
        let file_path = get_config_file_path();

        // Create the directory if it doesn't exist
        if let Some(parent_dir) = file_path.parent() {
            fs::create_dir_all(parent_dir).expect("Failed to create directory");
        }

        // Create the file if it doesn't exist
        if !file_path.exists() {
            let init_config = Config {
                commands: HashMap::new(),
                envs: HashMap::new(),
            };

            init_config.save().expect("could not save initial config")
        }

        let file = fs::File::open(file_path).expect("config file missing");
        let reader = BufReader::new(file);

        let config: Config = serde_json::from_reader(reader).expect("Error while reading JSON");
        return config;
    }
    pub fn save(&self) -> Result<(), std::io::Error> {
        let file_path = get_config_file_path();
        let mut file = fs::File::create(&file_path).expect("Failed to create file");

        file.write_all(serde_json::to_string_pretty(&self).unwrap().as_bytes())
    }
}
