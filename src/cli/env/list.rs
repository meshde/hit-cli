use crate::core::env::{get_env, list_envs};
use colored::Colorize;

pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    let envs = list_envs();
    let current_env = get_env();

    for env in envs {
        if current_env.clone().is_some_and(|e| env == e) {
            println!(" * {}", env.green());
        } else {
            println!("   {}", env);
        }
    }

    Ok(())
}
