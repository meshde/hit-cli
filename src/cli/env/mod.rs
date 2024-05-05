mod r#use;

use clap::Subcommand;
use std::error::Error;

#[derive(Subcommand, Debug)]
pub enum EnvCommand {
    Use(r#use::EnvUseArguments),
}

pub fn init(command: EnvCommand) -> Result<(), Box<dyn Error>> {
    match command {
        EnvCommand::Use(args) => r#use::init(args),
    }
}
