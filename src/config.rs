use crate::http;
use regex::Regex;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize, Clone)]
pub struct Command {
    pub method: http::HttpMethod,
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl Command {
    pub fn route_params(&self) -> Vec<String> {
        let route_param_regex = Regex::new(r"\/:(\w+)").unwrap();
        route_param_regex
            .captures_iter(self.url.as_str())
            .filter_map(|caps| caps.get(1))
            .map(|word| word.as_str().to_string())
            .collect::<HashSet<String>>()
            .into_iter()
            .collect()
    }
}

#[derive(Deserialize)]
pub struct Config {
    commands: HashMap<String, Command>,
    pub envs: HashMap<String, HashMap<String, String>>,
}

impl Config {
    pub fn new() -> Config {
        let file = File::open(".hitconfig.json").expect("config file missing");
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
