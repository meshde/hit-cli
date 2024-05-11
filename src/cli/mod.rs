mod env;
mod run;

use crate::config::Config;
use clap::{command, Arg, Command, FromArgMatches as _, Parser, Subcommand};
use convert_case::{Case, Casing};
use std::collections::HashMap;
use std::process::ExitCode;

#[derive(Debug, Parser)]
enum StaticCommand {
    #[command(subcommand)]
    Env(env::EnvCommand),
}

fn get_run_command() -> Command {
    let config = Config::new();

    let mut command = Command::new("run").arg_required_else_help(true);

    for command_name in config.commands() {
        let config_command = config.get_command(&command_name);
        let route_params = config_command.route_params();
        let mut subcommand =
            Command::new(command_name).arg_required_else_help(!route_params.is_empty());
        for route_param in route_params {
            subcommand = subcommand.arg(
                Arg::new(route_param.to_string())
                    .long(&route_param.to_string().to_case(Case::Kebab))
                    .value_name(route_param.to_string())
                    .help(format!(
                        "Provide value for the route param :{}",
                        route_param
                    )),
            )
        }
        command = command.subcommand(subcommand)
    }
    command
}

pub async fn init() -> ExitCode {
    let cli = Command::new("hit")
        .arg_required_else_help(true)
        .subcommand(get_run_command());

    let cli = StaticCommand::augment_subcommands(cli);

    let matches = cli.get_matches();

    let output = match matches.subcommand_name().unwrap() {
        "run" => {
            let run_subcommand_matches = matches.subcommand_matches("run").unwrap();

            let run_subcommand_name = run_subcommand_matches.subcommand_name().unwrap();
            let mut args_map = HashMap::new();

            let run_subcommand_matches = run_subcommand_matches
                .subcommand_matches(&run_subcommand_name)
                .unwrap();

            for arg_id in run_subcommand_matches.ids() {
                args_map.insert(
                    arg_id.to_string(),
                    run_subcommand_matches
                        .get_one::<String>(arg_id.as_str())
                        .unwrap()
                        .to_string(),
                );
            }
            run::run(run_subcommand_name.to_string(), args_map).await
        }
        _ => {
            let static_command_matches = StaticCommand::from_arg_matches(&matches).unwrap();

            match static_command_matches {
                StaticCommand::Env(args) => env::init(args),
            }
        }
    };

    if let Err(_e) = output {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
