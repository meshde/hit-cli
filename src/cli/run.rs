use crate::core::app_config::get_app_config;
use crate::core::command::Command;
use crate::core::config::Config;
use crate::core::env::get_env;
use crate::core::ephenv::get_ephenvs;
use crate::utils::error::CliError;
use crate::utils::http::handle_request;
use edit::edit;
use handlebars::Handlebars;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::io::stdout;
use std::io::Write;
use std::path::PathBuf;

const BREAKING_CHANGE_VERSION: &str = "0.6.0";

fn check_upgrade_notice() -> bool {
    let mut app_config = get_app_config();
    let needs_notice = match app_config.get_last_seen_version() {
        Some(version) => version.as_str() < BREAKING_CHANGE_VERSION,
        None => true,
    };

    if needs_notice {
        eprintln!(
            "NOTE: As of v{}, the editor no longer opens automatically for request bodies.",
            BREAKING_CHANGE_VERSION
        );
        eprintln!("Use --edit-body (-e) to review/edit the body before sending.");
        eprintln!("Please re-run your command to continue.");
        app_config.set_last_seen_version(env!("CARGO_PKG_VERSION").to_string());
    }

    needs_notice
}

fn replace_params(input: String, params: &HashMap<String, String>) -> String {
    params.keys().fold(input, |acc, x| {
        acc.replace(&format!(":{}", x), params.get(x).unwrap())
    })
}

pub struct RunOptions {
    pub edit_body: bool,
    pub body_file: Option<PathBuf>,
    pub json_output: bool,
}

struct TypedSubstitutionResult {
    body: String,
    file_fields: HashMap<String, PathBuf>,
}

fn replace_params_typed(
    input: String,
    params: &HashMap<String, String>,
    param_types: &HashMap<String, Option<String>>,
) -> Result<TypedSubstitutionResult, Box<dyn Error>> {
    let mut result = input.clone();
    let mut file_fields: HashMap<String, PathBuf> = HashMap::new();

    for (param_name, param_value) in params {
        let param_type = param_types
            .get(param_name.as_str())
            .and_then(|t| t.as_deref());

        match param_type {
            Some("boolean") => {
                if param_value != "true" && param_value != "false" {
                    return Err(Box::new(CliError {
                        message: format!(
                            "Parameter '{}' expects a boolean value (true/false), got '{}'",
                            param_name, param_value
                        ),
                    }));
                }
                let quoted_placeholder = format!("\":{}|boolean\"", param_name);
                result = result.replace(&quoted_placeholder, param_value);
            }
            Some("number") => {
                if param_value.parse::<f64>().is_err() {
                    return Err(Box::new(CliError {
                        message: format!(
                            "Parameter '{}' expects a number value, got '{}'",
                            param_name, param_value
                        ),
                    }));
                }
                let quoted_placeholder = format!("\":{}|number\"", param_name);
                result = result.replace(&quoted_placeholder, param_value);
            }
            Some("file") => {
                let path = PathBuf::from(param_value);
                if !path.exists() {
                    return Err(Box::new(CliError {
                        message: format!(
                            "File not found for parameter '{}': {}",
                            param_name, param_value
                        ),
                    }));
                }

                let placeholder = format!(":{}|file", param_name);
                if let Ok(mut json_val) = serde_json::from_str::<Value>(&result) {
                    if let Some(obj) = json_val.as_object_mut() {
                        let mut file_key = None;
                        for (key, val) in obj.iter() {
                            if let Some(s) = val.as_str() {
                                if s == placeholder {
                                    file_key = Some(key.clone());
                                    break;
                                }
                            }
                        }
                        if let Some(key) = file_key {
                            let key_clone = key.clone();
                            obj.remove(&key);
                            result = serde_json::to_string(&json_val).unwrap();
                            file_fields.insert(key_clone, path);
                        }
                    }
                }
            }
            _ => {
                let placeholder = format!(":{}", param_name);
                result = result.replace(&placeholder, param_value);
            }
        }
    }

    Ok(TypedSubstitutionResult {
        body: result,
        file_fields,
    })
}

pub async fn run(
    api_call: &Command,
    param_values: HashMap<String, String>,
    options: RunOptions,
) -> Result<(), Box<dyn Error>> {
    if check_upgrade_notice() {
        return Ok(());
    }

    let config = Config::new();
    let env_var_regex = Regex::new(r"\{\{\w+}}").unwrap();

    let hb_handle = Handlebars::new();

    let url = api_call.url.as_str();

    let current_env = match get_env() {
        Some(e) => e,
        None => {
            return Err(Box::new(CliError {
                message: "env not set".to_string(),
            }))
        }
    };
    let env_data = match config.envs.get(&current_env) {
        Some(d) => d,
        None => {
            return Err(Box::new(CliError {
                message: "env not recognized".to_string(),
            }))
        }
    };
    let ephenv_data = get_ephenvs();
    let merged_data = env_data
        .clone()
        .into_iter()
        .chain(ephenv_data.clone())
        .collect::<HashMap<String, String>>();

    let url_with_env_vars = if env_var_regex.is_match(url) {
        hb_handle.render_template(url, &merged_data).unwrap()
    } else {
        url.to_string()
    };

    let url_to_call = replace_params(url_with_env_vars, &param_values);

    let (input, file_fields) = if let Some(ref body_file_path) = options.body_file {
        let file_content = std::fs::read_to_string(body_file_path).map_err(|e| {
            Box::new(CliError {
                message: format!(
                    "Failed to read body file '{}': {}",
                    body_file_path.display(),
                    e
                ),
            })
        })?;
        let rendered = if env_var_regex.is_match(&file_content) {
            hb_handle.render_template(&file_content, &merged_data)?
        } else {
            file_content
        };
        (Some(rendered), None)
    } else if api_call.body.is_some() {
        let serialized = serde_json::to_string_pretty(&api_call.body).unwrap();
        let rendered = hb_handle
            .render_template(&serialized, &merged_data)
            .unwrap();

        let param_types = api_call.body_param_types();
        let typed_result = replace_params_typed(rendered, &param_values, &param_types)?;

        let body_str = if options.edit_body {
            edit(&typed_result.body).expect("Unable to open system editor")
        } else {
            typed_result.body
        };

        let file_fields = if typed_result.file_fields.is_empty() {
            None
        } else {
            Some(typed_result.file_fields)
        };

        (Some(body_str), file_fields)
    } else {
        (None, None)
    };

    let response = handle_request(
        url_to_call,
        &api_call.method,
        &api_call
            .headers
            .clone()
            .into_iter()
            .map(|(k, v)| (k, hb_handle.render_template(&v, &merged_data).unwrap()))
            .collect::<HashMap<String, String>>(),
        input,
        file_fields,
    )
    .await?;

    get_app_config().set_prev_request(response.clone());

    if options.json_output {
        println!("{}", serde_json::to_string(&response).unwrap());
    } else {
        let response_json_result = serde_json::from_str::<Value>(response.clone().body.as_str());

        match response_json_result {
            Ok(response_json) => {
                let mut out = stdout();
                colored_json::write_colored_json(&response_json, &mut out).unwrap();
                out.flush().unwrap();
                writeln!(out).unwrap();
            }
            Err(_error) => {
                println!("{}", response.body);
            }
        };
    }

    let mut postscript_env_vars = merged_data.clone();
    postscript_env_vars.extend(param_values);

    api_call
        .run_post_command_script(
            &serde_json::to_string_pretty(&response.clone()).unwrap(),
            &postscript_env_vars,
        )
        .unwrap();

    Ok(())
}
