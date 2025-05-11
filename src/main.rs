mod cli;
mod constants;
mod core;
mod utils;

use human_panic;
use std::process;
use tokio;

#[tokio::main]
async fn main() -> process::ExitCode {
    human_panic::setup_panic!();
    cli::init().await
}
