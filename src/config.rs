use crate::http;
use array_tool::vec::Union;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufReader, Write};
use std::path::PathBuf;

const CONFIG_DIR: &str = ".hit";

#[derive(Deserialize, Serialize, Clone)]
pub struct Command {
    pub method: http::HttpMethod,
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub body: Option<Value>,
}

fn get_params_from_string(input: &str) -> Vec<String> {
    let route_param_regex = Regex::new(r":(\w+)").unwrap();
    route_param_regex
        .captures_iter(input)
        .filter_map(|caps| caps.get(1))
        .map(|word| word.as_str().to_string())
        .collect::<HashSet<String>>()
        .into_iter()
        .collect()
}

impl Command {
    pub fn route_params(&self) -> Vec<String> {
        get_params_from_string(self.url.as_str())
    }

    pub fn body_params(&self) -> Vec<String> {
        match &self.body {
            Some(input) => get_params_from_string(&input.to_string()),
            None => Vec::new(),
        }
    }

    pub fn params(&self) -> Vec<String> {
        self.route_params().union(self.body_params())
    }
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub envs: HashMap<String, HashMap<String, String>>,
    commands: HashMap<String, Command>,
}

impl Config {
    pub fn new() -> Config {
        let file_path = PathBuf::from(CONFIG_DIR).join("config.json");

        // Create the directory if it doesn't exist
        if let Some(parent_dir) = file_path.parent() {
            fs::create_dir_all(parent_dir).expect("Failed to create directory");
        }

        // Create the file if it doesn't exist
        if !file_path.exists() {
            let mut file = fs::File::create(&file_path).expect("Failed to create file");
            let init_config = Config {
                commands: HashMap::new(),
                envs: HashMap::new(),
            };

            file.write_all(
                serde_json::to_string_pretty(&init_config)
                    .unwrap()
                    .as_bytes(),
            )
            .expect("could not save initial config")
        }

        let file = fs::File::open(file_path).expect("config file missing");
        let reader = BufReader::new(file);

        let config: Config = serde_json::from_reader(reader).expect("Error while reading JSON");
        return config;
    }

    pub fn commands(&self) -> Vec<String> {
        self.commands.keys().map(|key| key.clone()).collect()
    }

    pub fn get_command(&self, command: &String) -> Command {
        self.commands
            .get(command)
            .expect(&format!("Command not recognized: {}", command))
            .clone()
    }
}
