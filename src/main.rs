use reqwest;
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::io::stdout;
use tokio;
use colored_json;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let file = File::open(".hitconfig.json").expect("config file missing");
    let reader = BufReader::new(file);

    let config: Value = serde_json::from_reader(reader).expect("Error while reading JSON");

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

        if api_call.get("method").unwrap().as_str().unwrap() == "GET" {
            let response: Value = serde_json::from_str(reqwest::get(api_call.get("url").unwrap().as_str().unwrap()).await?.text().await?.as_str()).unwrap();
            let mut out = stdout();
            colored_json::write_colored_json(&response, &mut out).unwrap();
            writeln!(out, "").unwrap();
            out.flush().unwrap();
        } else {
            println!("Not recognised method");
        }
    } else {
        println!("command not recongnized");
    }

    Ok(())
}
