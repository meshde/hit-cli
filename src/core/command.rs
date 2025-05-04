use crate::constants::CONFIG_DIR;
use crate::utils::http;
use array_tool::vec::Union;
use convert_case::{Case, Casing};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::io::{Error, Write};
use std::path::PathBuf;
use std::process::Command as StdCommand;
use tempfile::NamedTempFile;

#[derive(Deserialize, Serialize, Clone)]
pub struct PostScriptConfig {
    pub command: String,
    pub file: String,
}

#[derive(Deserialize, Serialize, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use serde_json::json;

    #[fixture]
    fn input_command() -> Command {
        Command {
            method: http::HttpMethod::POST,
            url: String::from("https://example.com/orgs/:orgId/employees/:employeeId"),
            headers: HashMap::new(),
            body: Some(json!({
                "name": ":employeeName",
                "title": ":title",
            })),
            postscript: None,
        }
    }

    #[rstest]
    fn test_route_params(input_command: Command) {
        let mut route_params = input_command.route_params();
        route_params.sort();
        assert_eq!(
            route_params,
            vec!["employeeId".to_string(), "orgId".to_string()]
        )
    }

    #[rstest]
    fn test_body_params(input_command: Command) {
        let mut body_params = input_command.body_params();
        body_params.sort();
        assert_eq!(
            body_params,
            vec!["employeeName".to_string(), "title".to_string()]
        )
    }

    #[rstest]
    fn test_all_params(input_command: Command) {
        let mut all_params = input_command.params();
        all_params.sort();
        assert_eq!(
            all_params,
            vec![
                "employeeId".to_string(),
                "employeeName".to_string(),
                "orgId".to_string(),
                "title".to_string()
            ]
        )
    }
}
