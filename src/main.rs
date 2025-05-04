mod app_config;
mod cli;
mod command;
mod config;
mod constants;
mod env;
mod ephenv;
mod http;
mod input;

use reqwest;
use tokio;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    cli::init().await;
    Ok(())
}
