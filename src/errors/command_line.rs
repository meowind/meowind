use super::MeowindError;
use crate::utils::colors::*;
use std::fmt;

pub struct CommandLineError {
    pub message: String,
    pub kind: CommandLineErrorKind,
}

impl CommandLineError {
    pub fn new<T: ToString>(kind: CommandLineErrorKind, message: T) -> CommandLineError {
        CommandLineError {
            message: message.to_string(),
            kind,
        }
    }
}

impl MeowindError for CommandLineError {
    fn to_string(&self) -> String {
        format!(
            "{RED}{BOLD}command line error{RESET}: {}: {}",
            self.kind, self.message
        )
    }
}

pub enum CommandLineErrorKind {
    InvalidArguments,
    FailedToReadFile,
}

impl fmt::Display for CommandLineErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            CommandLineErrorKind::InvalidArguments => "invalid arguments",
            CommandLineErrorKind::FailedToReadFile => "failed to read file",
        };
        write!(f, "{text}")
    }
}
