use crate::http;
use array_tool::vec::Union;
use convert_case::{Case, Casing};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufReader, Error, Write};
use std::path::PathBuf;
use std::process::Command as StdCommand;
use tempfile::NamedTempFile;

const CONFIG_DIR: &str = ".hit";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PostScriptConfig {
    pub command: String,
    pub file: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Command {
    pub method: http::HttpMethod,
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    pub body: Option<Value>,
    pub postscript: Option<PostScriptConfig>,
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

    pub fn run_post_command_script(
        &self,
        command_response: &str,
        env_vars: &HashMap<String, String>,
    ) -> Result<(), Error> {
        if let Some(postscript) = self.postscript.clone() {
            let script_path = PathBuf::from(CONFIG_DIR)
                .join("postscripts")
                .join(postscript.file);

            if script_path.exists() {
                let mut response_file = NamedTempFile::new()?;
                response_file
                    .write_all(command_response.as_bytes())
                    .expect("could not save response to temp file");

                let hit_response_path_var = "HIT_RESPONSE_PATH";
                let mut command = StdCommand::new(postscript.command);
                command
                    .arg(script_path)
                    .env(hit_response_path_var, response_file.path());

                for (env_var, env_var_value) in env_vars {
                    let postscript_var_name = format!("HIT_{}", env_var.to_case(Case::UpperSnake));
                    command.env(postscript_var_name, env_var_value);
                }

                command.status().expect("failed to run postscript");
            }
        }
        Ok(())
    }
}

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

fn get_file_path() -> PathBuf {
    PathBuf::from(CONFIG_DIR).join("config.json")
}

impl Config {
    pub fn new() -> Config {
        let file_path = get_file_path();
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
    pub fn save(&self) -> Result<(), Error> {
        let file_path = get_file_path();
        let mut file = fs::File::create(&file_path).expect("Failed to create file");

        file.write_all(serde_json::to_string_pretty(&self).unwrap().as_bytes())
    }
}
