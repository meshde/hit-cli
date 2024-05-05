use crate::env::set_env;
use clap::Args;

#[derive(Args, Debug)]
pub struct EnvUseArguments {
    env: String,
}

pub fn init(args: EnvUseArguments) -> Result<(), Box<dyn std::error::Error>> {
    Ok(set_env(args.env))
}
