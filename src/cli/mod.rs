mod run;

use clap::{Parser, Subcommand};
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "hit")]
#[command(about = "CLI tool for API testing")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(external_subcommand)]
    Run(Vec<String>),
}

pub async fn init() -> ExitCode {
    let args = Cli::parse();

    let output = match args.command {
        Commands::Run(args) => run::init(args).await,
    };

    if let Err(_e) = output {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
