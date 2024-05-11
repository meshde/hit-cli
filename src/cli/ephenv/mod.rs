mod set;

use clap::Subcommand;
use std::error::Error;

#[derive(Subcommand, Debug)]
pub enum EphenvCommand {
    Set(set::EphenvSetArguments),
}

pub fn init(command: EphenvCommand) -> Result<(), Box<dyn Error>> {
    match command {
        EphenvCommand::Set(args) => set::init(args),
    }
}
