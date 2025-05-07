mod fixtures;
use assert_cmd::prelude::*;
use fixtures::{get_hit_command_for_dir, temp_dir};
use rstest::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[rstest]
fn test_import_swagger(temp_dir: TempDir) {
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/swagger.yml"),
        temp_dir.path().join("swagger.yml"),
    )
    .unwrap();

    let mut cmd = get_hit_command_for_dir(&temp_dir.path());
    cmd.args(["import", "./swagger.yml"]);
    cmd.assert().success();

    let config_path = temp_dir.path().join(".hit").join("config.json");
    let reader = fs::File::open(config_path).unwrap();
    let hit_config: serde_json::Value = serde_json::from_reader(reader).unwrap();
    insta::assert_json_snapshot!(hit_config);
}
