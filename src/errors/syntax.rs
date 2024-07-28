use crate::{frontend::Location, utils::colors::*};
use std::{fmt, path::PathBuf};

use super::{context::MeowindErrorContext, MeowindError};

pub struct SyntaxError {
    kind: SyntaxErrorKind,
    message: String,
    context: Option<MeowindErrorContext>,
}

impl SyntaxError {
    pub fn new<T: ToString>(
        kind: SyntaxErrorKind,
        message: T,
        context: Option<MeowindErrorContext>,
    ) -> SyntaxError {
        SyntaxError {
            kind,
            message: message.to_string(),
            context,
        }
    }

    pub fn new_with_context<T: ToString>(
        kind: SyntaxErrorKind,
        message: T,
        loc: Location,
        ln_text: String,
        src_path: PathBuf,
    ) -> SyntaxError {
        let context = Some(MeowindErrorContext::new(loc, ln_text, src_path));
        SyntaxError::new(kind, message, context)
    }
}

impl MeowindError for SyntaxError {
    fn to_string(&self) -> String {
        let mut error_body = format!(
            "{RED}{BOLD}syntax error{RESET}: {}: {}",
            self.kind, self.message
        );

        if let Some(context) = &self.context {
            error_body = format!("{error_body}\n{context}");
        }

        return error_body;
    }
}

pub enum SyntaxErrorKind {
    ExpectedCharacter,
    UnexpectedCharacter,
    UnexpectedToken,
    InvalidToken,
}

impl fmt::Display for SyntaxErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            SyntaxErrorKind::ExpectedCharacter => "expected character",
            SyntaxErrorKind::UnexpectedCharacter => "unexpected character",
            SyntaxErrorKind::UnexpectedToken => "unexpected token",
            SyntaxErrorKind::InvalidToken => "invalid token",
        };

        write!(f, "{text}")
    }
}
