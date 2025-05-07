use crate::core::openapi::generate_config;
use clap::{Args, ValueHint};
use openapiv3::OpenAPI;
use std::fs;

#[derive(Args, Debug)]
pub struct ImportArguments {
    #[arg(value_hint = ValueHint::FilePath)]
    file: String,
}

pub fn init(args: ImportArguments) -> Result<(), Box<dyn std::error::Error>> {
    // Read the OpenAPI spec from a file
    let spec_content = fs::read_to_string(args.file)?;
    let spec: OpenAPI = serde_yaml::from_str(&spec_content)?;

    // Generate configuration
    let config = generate_config(&spec)?;

    config.save().expect("could not create config file");
    Ok(())
}
