use colored_json;
use convert_case::{Case, Casing};
use getopts;
use regex::Regex;
use reqwest;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::stdout;
use std::io::BufReader;
use std::io::Write;
use std::process;
use tokio;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
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

    let args: Vec<String> = env::args().collect();

    if commands.contains(&args[1].as_str()) {
        let command = args[1].as_str();
        let api_call = config.get(command).unwrap();

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

        let matches = match opts.parse(&args[2..]) {
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
        let response: Option<Value> = match http_method {
            "GET" => {
                let resp = reqwest::get(url_to_call).await?.text().await?;
                serde_json::from_str(resp.as_str()).unwrap()
            }
            _ => None,
        };

        match response {
            Some(value) => {
                let mut out = stdout();
                colored_json::write_colored_json(&value, &mut out).unwrap();
                out.flush().unwrap();
                writeln!(out, "").unwrap();
            }
            None => {
                println!("HTTP method not supported: {}", http_method);
                process::exit(1)
            }
        }
    }

    Ok(())
}
