use crate::http::{handle_delete, handle_get, handle_post, handle_put};
use crate::input::CustomAutocomplete;
use arboard::Clipboard;
use colored_json;
use convert_case::{Case, Casing};
use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal;
use flatten_json_object::Flattener;
use getopts;
use inquire::Text;
use regex::Regex;
use reqwest;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::stdout;
use std::io::BufReader;
use std::io::Write;
use std::process;

// #[derive(Args, Debug)]
// pub struct RunArguments {
//     #[command(flatten)]
//     args: Vec<String>,
// }

fn get_json_value_from_path<'a, 'b>(json: &'a Value, path: &'b str) -> Option<&'a Value> {
    json.pointer(format!("/{}", path.replace(".", "/")).as_str())
}

pub async fn init(args: Vec<String>) -> Result<(), reqwest::Error> {
    let file = File::open(".hitconfig.json").expect("config file missing");
    let reader = BufReader::new(file);

    let config: Value = serde_json::from_reader(reader).expect("Error while reading JSON");
    let route_param_regex = Regex::new(r"\/:(\w+)").unwrap();

    let commands: Vec<&str> = config
        .as_object()
        .unwrap()
        .keys()
        .map(|key| key.as_str())
        .collect();

    let run_command = args[0].as_str();
    let run_command_flags = &args[1..];
    if commands.contains(&run_command) {
        let api_call = config.get(run_command).unwrap();

        let url = api_call
            .get("url")
            .expect("command missing url")
            .as_str()
            .expect("url is not string");

        let route_params: Vec<&str> = route_param_regex
            .captures_iter(url)
            .filter_map(|caps| caps.get(1))
            .map(|word| word.as_str())
            .collect::<HashSet<&str>>()
            .into_iter()
            .collect();

        let mut opts = getopts::Options::new();
        for route_param in &route_params {
            opts.optopt(
                "",
                &route_param.to_case(Case::Kebab),
                &format!("the value for {}", route_param),
                "",
            );
        }

        let matches = match opts.parse(run_command_flags) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{}", e.to_string());
                process::exit(1);
            }
        };

        let mut param_values: HashMap<&str, String> = HashMap::new();

        for route_param in &route_params {
            let kebab_cased_param = &route_param.to_case(Case::Kebab);
            let param_value = match matches.opt_str(kebab_cased_param) {
                Some(p) => p,
                None => {
                    eprintln!("Missing required option: --{}", kebab_cased_param);
                    process::exit(1);
                }
            };

            param_values.insert(route_param, param_value);
        }

        let http_method = api_call.get("method").unwrap().as_str().unwrap();
        let url_to_call = route_params.iter().fold(url.to_string(), |acc, &x| {
            acc.replace(&format!(":{}", x), &param_values.get(x).unwrap())
        });

        let response: String = match http_method {
            "GET" => handle_get(url_to_call).await?,
            "POST" => handle_post(url_to_call).await?,
            "PUT" => handle_put(url_to_call).await?,
            "DELETE" => handle_delete(url_to_call).await?,
            _ => {
                println!("HTTP method not supported: {}", http_method);
                process::exit(1)
            }
        };

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
