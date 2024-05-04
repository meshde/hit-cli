mod cli;
mod config;
mod http;
mod input;

use reqwest;
use tokio;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    cli::init().await;
    Ok(())
}
