use crate::core::ephenv::set_ephenv;
use clap::Args;

#[derive(Args, Debug)]
pub struct EphenvSetArguments {
    key: String,
    value: String,
}

pub fn init(args: EphenvSetArguments) -> Result<(), Box<dyn std::error::Error>> {
    Ok(set_ephenv(args.key, args.value))
}
