use crate::core::env::set_env;
use clap::Args;

#[derive(Args, Debug)]
pub struct EnvUseArguments {
    env: String,
}

pub fn init(args: EnvUseArguments) -> Result<(), Box<dyn std::error::Error>> {
    set_env(args.env);
    Ok(())
}
