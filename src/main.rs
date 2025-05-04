mod cli;
mod constants;
mod core;
mod utils;

use reqwest;
use tokio;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    cli::init().await;
    Ok(())
}
