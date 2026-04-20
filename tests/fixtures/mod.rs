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
    pub fn new(temp_dir: TempDir) -> Self {
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
            "commands": {
                "get-by-id": {
                    "method": "GET",
                    "url": "{{API_URL}}/items/:id",
                }
            }
        });

        fs::write(&config_path, test_config.to_string()).unwrap();

        seed_app_config(&temp_dir);

        Self { temp_dir }
    }
}

#[fixture]
pub fn temp_dir() -> TempDir {
    TempDir::new_in(".").unwrap()
}

#[fixture]
pub fn hit_setup(temp_dir: TempDir) -> SetupFixture {
    SetupFixture::new(temp_dir)
}

fn seed_app_config(temp_dir: &TempDir) {
    let app_config_path = temp_dir.path().join("config.json");
    let app_config = serde_json::json!({
        "last_seen_version": env!("CARGO_PKG_VERSION")
    });
    fs::write(&app_config_path, app_config.to_string()).unwrap();
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

pub fn setup_with_mock(
    temp_dir: TempDir,
    server_url: &str,
    commands_json: serde_json::Value,
) -> SetupFixture {
    let config_path = temp_dir.path().join(".hit").join("config.json");
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();

    let test_config = serde_json::json!({
        "envs": {
            "dev": {
                "API_URL": server_url
            }
        },
        "commands": commands_json
    });

    fs::write(&config_path, test_config.to_string()).unwrap();

    seed_app_config(&temp_dir);

    let setup = SetupFixture { temp_dir };

    // Set env to "dev"
    let mut use_cmd = get_hit_command_for_setup(&setup);
    use_cmd.args(["env", "use", "dev"]);
    use_cmd
        .output()
        .expect("Failed to set env to dev in setup_with_mock");

    setup
}
