mod view;

use clap::Subcommand;
use std::error::Error;

#[derive(Subcommand, Debug)]
pub enum LastCommand {
    View,
}

pub fn init(command: LastCommand) -> Result<(), Box<dyn Error>> {
    match command {
        LastCommand::View => view::init(),
    }
}
