use std::fmt;

use super::{context::ErrorContext, MeowindError};
use crate::utils::colors::*;

#[derive(Clone)]
pub struct CompilerError<'a> {
    kind: Option<CompilerErrorKind>,
    msg: Option<String>,
    ctx: Option<ErrorContext<'a>>,
}

impl Default for CompilerError<'_> {
    fn default() -> Self {
        Self {
            kind: None,
            msg: None,
            ctx: None,
        }
    }
}

impl<'a> CompilerError<'a> {
    pub fn kind(&self, kind: CompilerErrorKind) -> Self {
        Self {
            kind: Some(kind),
            ..self.clone()
        }
    }

    pub fn msg<T: ToString>(&self, msg: T) -> Self {
        Self {
            msg: Some(msg.to_string()),
            ..self.clone()
        }
    }

    pub fn ctx(&self, ctx: ErrorContext<'a>) -> Self {
        Self {
            ctx: Some(ctx),
            ..self.clone()
        }
    }
}

impl MeowindError for CompilerError<'_> {
    fn to_string(&self) -> String {
        let mut error_body = format!("{RED}{BOLD}compiler error{RESET}");

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
pub enum CompilerErrorKind {
    Undeclared,
    AlreadyDeclared,
}

impl fmt::Display for CompilerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            Self::Undeclared => "undeclared",
            Self::AlreadyDeclared => "already declared",
        };

        write!(f, "{text}")
    }
}
