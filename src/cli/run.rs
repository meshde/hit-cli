use crate::core::app_config::get_app_config;
use crate::core::command::Command;
use crate::core::config::Config;
use crate::core::env::get_env;
use crate::core::ephenv::get_ephenvs;
use crate::utils::error::CliError;
use crate::utils::http::handle_request;
use colored_json;
use edit::edit;
use handlebars::Handlebars;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::io::stdout;
use std::io::Write;

fn replace_params(input: String, params: &HashMap<String, String>) -> String {
    params.keys().fold(input, |acc, x| {
        acc.replace(&format!(":{}", x), &params.get(x).unwrap())
    })
}

pub async fn run(
    api_call: &Command,
    param_values: HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    let config = Config::new();
    let env_var_regex = Regex::new(r"\{\{\w+}}").unwrap();

    let hb_handle = Handlebars::new();

    let url = api_call.url.as_str();

    let current_env = match get_env() {
        Some(e) => e,
        None => {
            return Err(Box::new(CliError {
                message: "env not set".to_string(),
                help: None,
            }))
        }
    };
    let env_data = match config.envs.get(&current_env) {
        Some(d) => d,
        None => {
            return Err(Box::new(CliError {
                message: "env not recognized".to_string(),
                help: None,
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

    let input = if api_call.body.is_some() {
        Some(
            edit(replace_params(
                hb_handle
                    .render_template(
                        &serde_json::to_string_pretty(&api_call.body).unwrap(),
                        &merged_data,
                    )
                    .unwrap(),
                &param_values,
            ))
            .expect("Unable to open system editor"),
        )
    } else {
        None
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
    )
    .await
    .unwrap();

    get_app_config().set_prev_request(response.clone());

    let response_json_result = serde_json::from_str::<Value>(response.clone().body.as_str());

    match response_json_result {
        Ok(response_json) => {
            let mut out = stdout();
            colored_json::write_colored_json(&response_json, &mut out).unwrap();
            out.flush().unwrap();
            writeln!(out, "").unwrap();
            let mut postscript_env_vars = merged_data.clone();
            postscript_env_vars.extend(param_values);

            api_call
                .run_post_command_script(
                    &serde_json::to_string_pretty(&response.clone()).unwrap(),
                    &postscript_env_vars,
                )
                .unwrap();
        }
        Err(_error) => {
            println!("{}", response.body);
        }
    };

    Ok(())
}
