use crate::app_config::get_app_config;
use crate::input::CustomAutocomplete;
use arboard::Clipboard;
use colored_json;
use crossterm::event::{read, Event, KeyCode};
use crossterm::terminal;
use flatten_json_object::Flattener;
use inquire::Text;
use serde_json::Value;
use std::io::stdout;
use std::io::Write;

fn get_json_value_from_path<'a, 'b>(json: &'a Value, path: &'b str) -> Option<&'a Value> {
    json.pointer(format!("/{}", path.replace(".", "/")).as_str())
}

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let prev_request = get_app_config()
        .get_prev_request()
        .expect("No last request found")
        .clone();

    let mut prev_request = serde_json::to_value(prev_request).unwrap();
    if let Ok(body_json) = serde_json::from_str::<Value>(&prev_request["body"].as_str().unwrap()) {
        prev_request["body"] = body_json;
    }

    let mut out = stdout();
    colored_json::write_colored_json(&prev_request, &mut out).unwrap();
    out.flush().unwrap();
    writeln!(out, "").unwrap();
    println!("Press c to enter copy mode or any other key  to exit");
    terminal::enable_raw_mode().unwrap();

    loop {
        if let Ok(event) = read() {
            if let Event::Key(key) = event {
                terminal::disable_raw_mode().unwrap();
                if key.code == KeyCode::Char('c') {
                    let flattened_json = Flattener::new().flatten(&prev_request).unwrap();

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
                            get_json_value_from_path(&prev_request, &user_json_path)
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
    Ok(())
}
