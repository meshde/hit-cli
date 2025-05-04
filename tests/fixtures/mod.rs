use assert_cmd::prelude::*;
use rstest::*;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[derive(Debug)]
pub struct SetupFixture {
    pub temp_dir: TempDir,
}

impl SetupFixture {
    pub fn new() -> Self {
        let temp_dir = TempDir::new_in(".").unwrap();
        let config_path = temp_dir.path().join(".hit").join("config.json");

        fs::create_dir_all(config_path.parent().unwrap()).unwrap();

        let test_config = serde_json::json!({
            "envs": {
                "prod": {
                    "API_URL": "https://api.example.com"
                },
                "dev": {
                    "API_URL": "https://dev-api.example.com"
                },
                "staging": {
                    "API_URL": "https://staging-api.example.com"
                }
            },
            "commands": {}
        });

        fs::write(&config_path, test_config.to_string()).unwrap();

        Self { temp_dir }
    }
}

#[fixture]
pub fn hit_setup() -> SetupFixture {
    SetupFixture::new()
}

pub fn get_hit_command_for_dir(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("hit-cli").expect("could not call hit-cli");
    cmd.current_dir(dir);
    return cmd;
}

pub fn get_hit_command_for_setup(setup: &SetupFixture) -> Command {
    let app_config_dir = setup.temp_dir.path().to_string_lossy().to_string();
    let mut cmd = get_hit_command_for_dir(setup.temp_dir.path());
    cmd.env("APP_CONFIG_DIR", app_config_dir);
    return cmd;
}
