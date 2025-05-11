mod fixtures;
use assert_cmd::prelude::*;
use fixtures::{get_hit_command_for_setup, hit_setup, SetupFixture};
use rstest::*;

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
