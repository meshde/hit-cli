mod env;
mod ephenv;
mod import;
mod last;
mod run;

use crate::core::command::Command as ConfigCommand;
use crate::core::config::{CommandType as ConfigCommandType, Config};
use crate::utils::error::CliError;
use clap::{command, Arg, ArgMatches, Command, FromArgMatches as _, Parser, Subcommand};
use clap_complete::CompleteEnv;
use convert_case::{Case, Casing};
use std::collections::HashMap;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(version)]
enum StaticCommand {
    #[command(subcommand)]
    Env(env::EnvCommand),
    #[command(subcommand)]
    Ephenv(ephenv::EphenvCommand),
    #[command(subcommand)]
    Last(last::LastCommand),
    Import(import::ImportArguments),
}

fn formulate_command(
    mut command: Command,
    config_commands: &HashMap<String, Box<ConfigCommandType>>,
) -> Command {
    for (key, value) in config_commands.iter() {
        let subcommand = match **value {
            ConfigCommandType::Command(ref config_command) => {
                let params = config_command.params();

                let mut subcommand = Command::new(key).arg_required_else_help(!params.is_empty());
                for param in params {
                    subcommand = subcommand.arg(
                        Arg::new(param.to_string())
                            .long(&param.to_string().to_case(Case::Kebab))
                            .value_name(param.to_string())
                            .help(format!("Provide value for the param :{}", param)),
                    )
                }

                subcommand
            }
            ConfigCommandType::NestedCommand(ref nested_commands) => formulate_command(
                Command::new(key).arg_required_else_help(true),
                nested_commands,
            ),
        };
        command = command.clone().subcommand(subcommand);
    }
    command.clone()
}

fn obtain_run_command_from_matches(
    matches: &ArgMatches,
    config_commands: &HashMap<String, Box<ConfigCommandType>>,
    args_map: &mut HashMap<String, String>,
) -> ConfigCommand {
    let subcommand_name = matches.subcommand_name().unwrap();
    let config_command_value = config_commands.get(subcommand_name).unwrap();
    let subcommand_matches = matches.subcommand_matches(&subcommand_name).unwrap();

    match **config_command_value {
        ConfigCommandType::Command(ref config_command) => {
            for arg_id in subcommand_matches.ids() {
                args_map.insert(
                    arg_id.to_string(),
                    subcommand_matches
                        .get_one::<String>(arg_id.as_str())
                        .unwrap()
                        .to_string(),
                );
            }
            config_command.clone()
        }
        ConfigCommandType::NestedCommand(ref config_command) => {
            obtain_run_command_from_matches(&subcommand_matches, &config_command, args_map)
        }
    }
}

fn get_run_command(config: &Config) -> Command {
    let mut command = Command::new("run").arg_required_else_help(true);

    command = formulate_command(command, &config.commands);
    command
}

pub async fn init() -> ExitCode {
    let config = Config::new();

    let cli = Command::new("hit")
        .arg_required_else_help(true)
        .subcommand(get_run_command(&config));

    let cli = StaticCommand::augment_subcommands(cli);

    CompleteEnv::with_factory(|| cli.clone()).complete();

    let matches = cli.get_matches();

    let output = match matches.subcommand_name().unwrap() {
        "run" => {
            let run_subcommand_matches = matches.subcommand_matches("run").unwrap();

            let mut args_map = HashMap::new();

            let config_command = obtain_run_command_from_matches(
                &run_subcommand_matches,
                &config.commands,
                &mut args_map,
            );
            run::run(&config_command, args_map).await
        }
        _ => {
            let static_command_matches = StaticCommand::from_arg_matches(&matches).unwrap();

            match static_command_matches {
                StaticCommand::Env(args) => env::init(args),
                StaticCommand::Ephenv(args) => ephenv::init(args),
                StaticCommand::Last(args) => last::init(args),
                StaticCommand::Import(args) => import::init(args),
            }
        }
    };

    if let Err(_e) = output {
        if let Some(e) = _e.downcast_ref::<CliError>() {
            eprintln!("{}", e);
        } else {
            panic!("{}", _e);
        }
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
