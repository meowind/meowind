use crate::{
    frontend::{lexing::Token, Loc},
    structs::ScriptSource,
    utils::colors::*,
};
use std::{fmt, path::PathBuf};

use super::{
    context::{ErrorContext, ErrorContextKind},
    MeowindError,
};

#[derive(Clone)]
pub struct SyntaxError {
    kind: Option<SyntaxErrorKind>,
    msg: Option<String>,
    ctx: Option<ErrorContext>,
}

impl Default for SyntaxError {
    fn default() -> Self {
        Self {
            kind: None,
            msg: None,
            ctx: None,
        }
    }
}

impl SyntaxError {
    pub fn kind(&self, kind: SyntaxErrorKind) -> SyntaxError {
        SyntaxError {
            kind: Some(kind),
            ..self.clone()
        }
    }

    pub fn msg<T: ToString>(&self, msg: T) -> SyntaxError {
        SyntaxError {
            msg: Some(msg.to_string()),
            ..self.clone()
        }
    }

    pub fn ctx(&self, ctx: ErrorContext) -> SyntaxError {
        SyntaxError {
            ctx: Some(ctx),
            ..self.clone()
        }
    }
}

impl MeowindError for SyntaxError {
    fn to_string(&self) -> String {
        let mut error_body = format!("{RED}{BOLD}syntax error{RESET}");

        if let Some(kind) = &self.kind {
            error_body += format!(": {kind}").as_str();
        }

        if let Some(msg) = &self.msg {
            error_body += format!(": {msg}").as_str();
        }

        if let Some(ctx) = &self.ctx {
            error_body = format!("{error_body}\n{}", ctx.to_string());
        }

        return error_body;
    }
}

#[derive(Clone)]
pub enum SyntaxErrorKind {
    Expected(SyntaxErrorSource),
    Unexpected(SyntaxErrorSource),
    Invalid(SyntaxErrorSource),
}

#[derive(Debug, Clone)]
pub enum SyntaxErrorSource {
    Character,
    Token,
    Expression,
}

impl ToString for SyntaxErrorSource {
    fn to_string(&self) -> String {
        return format!("{:?}", self);
    }
}

impl fmt::Display for SyntaxErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            SyntaxErrorKind::Expected(source) => {
                format!("expected {}", source.to_string().to_lowercase())
            }
            SyntaxErrorKind::Unexpected(source) => {
                format!("unexpected {}", source.to_string().to_lowercase())
            }
            SyntaxErrorKind::Invalid(source) => {
                format!("invalid {}", source.to_string().to_lowercase())
            }
        };

        write!(f, "{text}")
    }
}
