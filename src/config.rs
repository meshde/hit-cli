use crate::http;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize)]
pub struct Command {
    pub method: http::HttpMethod,
    pub url: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub commands: HashMap<String, Command>,
    pub envs: HashMap<String, HashMap<String, String>>,
}

impl Config {
    pub fn new() -> Config {
        let file = File::open(".hitconfig.json").expect("config file missing");
        let reader = BufReader::new(file);

        let config: Config = serde_json::from_reader(reader).expect("Error while reading JSON");
        return config;
    }
}
