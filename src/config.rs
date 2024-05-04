use crate::http;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Command {
    pub method: http::HttpMethod,
    pub url: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub commands: HashMap<String, Command>,
}
