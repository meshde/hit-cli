mod cli;
mod constants;
mod core;
mod utils;

use std::process;

#[tokio::main]
async fn main() -> process::ExitCode {
    human_panic::setup_panic!(human_panic::Metadata::new("hit", env!("CARGO_PKG_VERSION"))
        .authors("Mehmood S. Deshmukh <meshde.md@gmail.com>")
        .homepage("https://github.com/meshde/hit-cli/issues"));
    cli::init().await
}
