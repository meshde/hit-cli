mod env;
mod run;

use clap::{command, Arg, Command, FromArgMatches as _, Parser, Subcommand};
use std::process::ExitCode;

#[derive(Debug, Parser)]
enum StaticCommand {
    #[command(subcommand)]
    Env(env::EnvCommand),
}

pub async fn init() -> ExitCode {
    let cli =
        Command::new("hit").subcommand(Command::new("run").arg(Arg::new("command").num_args(1..)));

    let cli = StaticCommand::augment_subcommands(cli);
    let output;

    let matches = cli.get_matches();

    if matches.subcommand_name() == Some("run") {
        output = run::run(
            matches
                .subcommand_matches("run")
                .unwrap()
                .get_raw("command")
                .unwrap()
                .into_iter()
                .map(|x| x.to_string_lossy().to_string())
                .collect(),
        )
        .await;
    } else {
        let static_command_matches = StaticCommand::from_arg_matches(&matches).unwrap();

        output = match static_command_matches {
            StaticCommand::Env(args) => env::init(args),
        };
    }

    if let Err(_e) = output {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
