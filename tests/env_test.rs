mod fixtures;
use assert_cmd::prelude::*;
use fixtures::{get_hit_command_for_setup, hit_setup, SetupFixture};
use rstest::*;

#[rstest]
fn test_env_list(hit_setup: SetupFixture) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = get_hit_command_for_setup(&hit_setup);
    cmd.args(["env", "list"]);

    cmd.assert()
        .success()
        .stdout("   dev\n   prod\n   staging\n");

    Ok(())
}

#[rstest]
fn test_env_use(hit_setup: SetupFixture) -> Result<(), Box<dyn std::error::Error>> {
    let mut use_cmd = get_hit_command_for_setup(&hit_setup);
    use_cmd.args(["env", "use", "prod"]);
    use_cmd.assert().success();

    let mut list_cmd = get_hit_command_for_setup(&hit_setup);
    list_cmd.args(["env", "list"]);
    list_cmd
        .assert()
        .success()
        .stdout("   dev\n * prod\n   staging\n");

    Ok(())
}
