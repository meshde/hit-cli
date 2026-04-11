mod fixtures;
use assert_cmd::prelude::*;
use fixtures::{get_hit_command_for_setup, hit_setup, setup_with_mock, temp_dir, SetupFixture};
use predicates::prelude::*;
use rstest::*;
use tempfile::TempDir;

#[rstest]
fn test_failure_when_env_not_set(
    hit_setup: SetupFixture,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = get_hit_command_for_setup(&hit_setup);
    cmd.args(["run", "get-by-id", "--id", "meshde"]);
    cmd.assert().failure().stderr("env not set\n");

    Ok(())
}

#[rstest]
fn test_failure_when_env_not_recognized(hit_setup: SetupFixture) -> () {
    let mut use_cmd = get_hit_command_for_setup(&hit_setup);
    use_cmd.args(["env", "use", "something"]);
    use_cmd.assert().success();

    let mut cmd = get_hit_command_for_setup(&hit_setup);
    cmd.args(["run", "get-by-id", "--id", "meshde"]);
    cmd.assert().failure().stderr("env not recognized\n");
}

// --- Feature D: Typed Parameter Substitution ---

#[rstest]
fn test_typed_boolean_param(temp_dir: TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/endpoint")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "dryRun": true,
            "name": "John"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"result":"ok"}"#)
        .create();

    let commands = serde_json::json!({
        "cmd": {
            "method": "POST",
            "url": "{{API_URL}}/endpoint",
            "headers": {
                "Content-Type": "application/json"
            },
            "body": {
                "dryRun": ":dryRun|boolean",
                "name": ":name"
            }
        }
    });

    let setup = setup_with_mock(temp_dir, &server.url(), commands);
    let mut cmd = get_hit_command_for_setup(&setup);
    cmd.args([
        "run",
        "cmd",
        "--dry-run",
        "true",
        "--name",
        "John",
        "--json",
    ]);
    cmd.assert().success();

    mock.assert();
    Ok(())
}

#[rstest]
fn test_typed_number_param(temp_dir: TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/endpoint")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "limit": 42
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"result":"ok"}"#)
        .create();

    let commands = serde_json::json!({
        "cmd": {
            "method": "POST",
            "url": "{{API_URL}}/endpoint",
            "headers": {
                "Content-Type": "application/json"
            },
            "body": {
                "limit": ":limit|number"
            }
        }
    });

    let setup = setup_with_mock(temp_dir, &server.url(), commands);
    let mut cmd = get_hit_command_for_setup(&setup);
    cmd.args(["run", "cmd", "--limit", "42", "--json"]);
    cmd.assert().success();

    mock.assert();
    Ok(())
}

#[rstest]
fn test_typed_boolean_validation_error(
    temp_dir: TempDir,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let _mock = server.mock("POST", "/endpoint").with_status(200).create();

    let commands = serde_json::json!({
        "cmd": {
            "method": "POST",
            "url": "{{API_URL}}/endpoint",
            "headers": {
                "Content-Type": "application/json"
            },
            "body": {
                "dryRun": ":dryRun|boolean"
            }
        }
    });

    let setup = setup_with_mock(temp_dir, &server.url(), commands);
    let mut cmd = get_hit_command_for_setup(&setup);
    cmd.args(["run", "cmd", "--dry-run", "notabool"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("boolean"));

    Ok(())
}

#[rstest]
fn test_typed_number_validation_error(temp_dir: TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let _mock = server.mock("POST", "/endpoint").with_status(200).create();

    let commands = serde_json::json!({
        "cmd": {
            "method": "POST",
            "url": "{{API_URL}}/endpoint",
            "headers": {
                "Content-Type": "application/json"
            },
            "body": {
                "limit": ":limit|number"
            }
        }
    });

    let setup = setup_with_mock(temp_dir, &server.url(), commands);
    let mut cmd = get_hit_command_for_setup(&setup);
    cmd.args(["run", "cmd", "--limit", "notanumber"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("number"));

    Ok(())
}

// --- Feature A: Multipart File Uploads ---

#[rstest]
fn test_file_upload_multipart(temp_dir: TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/upload")
        .match_header(
            "content-type",
            mockito::Matcher::Regex("multipart/form-data.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"result":"uploaded"}"#)
        .create();

    let commands = serde_json::json!({
        "cmd": {
            "method": "POST",
            "url": "{{API_URL}}/upload",
            "headers": {
                "Content-Type": "application/json"
            },
            "body": {
                "file": ":filePath|file",
                "dryRun": ":dryRun|boolean"
            }
        }
    });

    // Create a temp CSV file
    let csv_path = temp_dir.path().join("test.csv");
    std::fs::write(&csv_path, "col1,col2\nval1,val2\n")?;

    let setup = setup_with_mock(temp_dir, &server.url(), commands);
    let mut cmd = get_hit_command_for_setup(&setup);
    cmd.args([
        "run",
        "cmd",
        "--file-path",
        csv_path.to_str().unwrap(),
        "--dry-run",
        "true",
        "--json",
    ]);
    cmd.assert().success();

    mock.assert();
    Ok(())
}

#[rstest]
fn test_file_not_found_error(temp_dir: TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let _mock = server.mock("POST", "/upload").with_status(200).create();

    let commands = serde_json::json!({
        "cmd": {
            "method": "POST",
            "url": "{{API_URL}}/upload",
            "headers": {
                "Content-Type": "application/json"
            },
            "body": {
                "file": ":filePath|file"
            }
        }
    });

    let setup = setup_with_mock(temp_dir, &server.url(), commands);
    let mut cmd = get_hit_command_for_setup(&setup);
    cmd.args(["run", "cmd", "--file-path", "/nonexistent/path.csv"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));

    Ok(())
}

// --- Feature B: Non-Interactive Body Handling ---

#[rstest]
fn test_no_editor_by_default(temp_dir: TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/endpoint")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "name": "John"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"result":"ok"}"#)
        .create();

    let commands = serde_json::json!({
        "cmd": {
            "method": "POST",
            "url": "{{API_URL}}/endpoint",
            "headers": {
                "Content-Type": "application/json"
            },
            "body": {
                "name": ":name"
            }
        }
    });

    let setup = setup_with_mock(temp_dir, &server.url(), commands);
    let mut cmd = get_hit_command_for_setup(&setup);
    cmd.args(["run", "cmd", "--name", "John", "--json"]);
    cmd.assert().success();

    mock.assert();
    Ok(())
}

#[rstest]
fn test_body_file_flag(temp_dir: TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("POST", "/endpoint")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "custom": "payload"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"result":"ok"}"#)
        .create();

    let commands = serde_json::json!({
        "cmd": {
            "method": "POST",
            "url": "{{API_URL}}/endpoint",
            "headers": {
                "Content-Type": "application/json"
            },
            "body": {
                "name": ":name"
            }
        }
    });

    // Create a temp body file
    let body_path = temp_dir.path().join("body.json");
    std::fs::write(&body_path, r#"{"custom": "payload"}"#)?;

    let setup = setup_with_mock(temp_dir, &server.url(), commands);
    let mut cmd = get_hit_command_for_setup(&setup);
    cmd.args([
        "run",
        "cmd",
        "--body-file",
        body_path.to_str().unwrap(),
        "--json",
    ]);
    cmd.assert().success();

    mock.assert();
    Ok(())
}

#[rstest]
fn test_body_file_not_found(temp_dir: TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let _mock = server.mock("POST", "/endpoint").with_status(200).create();

    let commands = serde_json::json!({
        "cmd": {
            "method": "POST",
            "url": "{{API_URL}}/endpoint",
            "headers": {
                "Content-Type": "application/json"
            },
            "body": {
                "name": ":name"
            }
        }
    });

    let setup = setup_with_mock(temp_dir, &server.url(), commands);
    let mut cmd = get_hit_command_for_setup(&setup);
    cmd.args([
        "run",
        "cmd",
        "--body-file",
        "/nonexistent/body.json",
        "--json",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to read body file"));

    Ok(())
}

// --- Feature C: Structured JSON Output ---

#[rstest]
fn test_json_output_structure(temp_dir: TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/items/123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"result":"ok"}"#)
        .create();

    let commands = serde_json::json!({
        "cmd": {
            "method": "GET",
            "url": "{{API_URL}}/items/:id"
        }
    });

    let setup = setup_with_mock(temp_dir, &server.url(), commands);
    let mut cmd = get_hit_command_for_setup(&setup);
    cmd.args(["run", "cmd", "--id", "123", "--json"]);

    let output = cmd.output()?;
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    let json_response: serde_json::Value = serde_json::from_str(&stdout)?;

    assert!(json_response.get("url").is_some());
    assert!(json_response.get("status").is_some());
    assert!(json_response.get("headers").is_some());
    assert!(json_response.get("body").is_some());
    assert_eq!(json_response["status"], 200);

    mock.assert();
    Ok(())
}

#[rstest]
fn test_default_output_not_json_structured(
    temp_dir: TempDir,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/items/123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"result":"ok"}"#)
        .create();

    let commands = serde_json::json!({
        "cmd": {
            "method": "GET",
            "url": "{{API_URL}}/items/:id"
        }
    });

    let setup = setup_with_mock(temp_dir, &server.url(), commands);
    let mut cmd = get_hit_command_for_setup(&setup);
    cmd.args(["run", "cmd", "--id", "123"]);

    let output = cmd.output()?;
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout)?;
    // Without --json, output should NOT be the structured JSON with url/status/headers/body
    assert!(!stdout.starts_with("{\"url\":"));

    mock.assert();
    Ok(())
}
