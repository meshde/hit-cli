use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct CliError {
    pub message: String,
    pub help: Option<String>,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl Error for CliError {}
