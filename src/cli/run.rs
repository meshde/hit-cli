use crate::config::Config;
use crate::env::get_env;
use crate::http::handle_request;
use crate::input::CustomAutocomplete;
use arboard::Clipboard;
use colored_json;
use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal;
use flatten_json_object::Flattener;
use handlebars::Handlebars;
use inquire::Text;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::io::stdout;
use std::io::Write;

fn get_json_value_from_path<'a, 'b>(json: &'a Value, path: &'b str) -> Option<&'a Value> {
    json.pointer(format!("/{}", path.replace(".", "/")).as_str())
}

pub async fn run(
    run_command: String,
    param_values: HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    let config = Config::new();
    let env_var_regex = Regex::new(r"\{\{\w+}}").unwrap();

    let hb_handle = Handlebars::new();

    if config.commands().contains(&run_command) {
        let api_call = config.get_command(&run_command);

        let url = api_call.url.as_str();

        let route_params = api_call.route_params();

        let url_with_env_vars = if env_var_regex.is_match(url) {
            let current_env = get_env().expect("env not set");
            let env_data = config.envs.get(&current_env).expect("env not recognized");
            hb_handle.render_template(url, env_data).unwrap()
        } else {
            url.to_string()
        };

        let url_to_call = route_params.iter().fold(url_with_env_vars, |acc, x| {
            acc.replace(&format!(":{}", x), &param_values.get(x).unwrap())
        });

        let response = handle_request(url_to_call, &api_call.method).await?;
        let response_json_result = serde_json::from_str::<Value>(response.as_str());

        match response_json_result {
            Ok(response_json) => {
                let mut out = stdout();
                colored_json::write_colored_json(&response_json, &mut out).unwrap();
                out.flush().unwrap();
                writeln!(out, "").unwrap();
                println!("Press c to enter copy mode or any other key  to exit");
                terminal::enable_raw_mode().unwrap();

                loop {
                    if let Ok(event) = read() {
                        if let Event::Key(key) = event {
                            terminal::disable_raw_mode().unwrap();
                            if key.code == KeyCode::Char('c') {
                                let flattened_json =
                                    Flattener::new().flatten(&response_json).unwrap();

                                let json_paths: Vec<String> = flattened_json
                                    .as_object()
                                    .unwrap()
                                    .keys()
                                    .map(|k| k.to_string())
                                    .collect();

                                let user_json_path = Text::new("Enter the JSON path: ")
                                    .with_autocomplete(CustomAutocomplete::new(json_paths))
                                    .prompt()
                                    .unwrap();
                                Clipboard::new()
                                    .unwrap()
                                    .set_text(
                                        get_json_value_from_path(&response_json, &user_json_path)
                                            .unwrap()
                                            .to_string(),
                                    )
                                    .unwrap();
                            }
                            break;
                        }
                    }
                }
                println!("");
            }
            Err(_error) => {
                println!("{}", response);
            }
        };
    } else {
        println!("Command not recognized: {}", run_command);
    }
    Ok(())
}
