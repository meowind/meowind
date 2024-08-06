use crate::{
    errors::{
        context::ErrorContext,
        syntax::{SyntaxError, SyntaxErrorKind, SyntaxErrorSource},
    },
    frontend::{
        lexing::{Assignments, Keywords, Punctuations, Tokens},
        parsing::ast::functions::{ArgumentNode, FunctionNode},
    },
    source::SourcePoint,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_function(&mut self) -> Result<FunctionNode, SyntaxError> {
        self.expect(Tokens::Keyword(Keywords::Func))?;

        self.advance();
        let name_token = self.expect(Tokens::Identifier)?;

        self.advance();
        let args = self.parse_function_arguments()?;

        self.advance();
        let mut r#type = None;
        let mut return_var = None;

        if self.current().kind == Tokens::Punctuation(Punctuations::ReturnSeparator) {
            self.advance();
            r#type = Some(self.parse_type()?);

            self.advance();
            if self.current().kind == Tokens::Punctuation(Punctuations::Colon) {
                return_var = Some(r#type.unwrap().raw);

                self.advance();
                r#type = Some(self.parse_type()?);

                self.advance();
            }
        }

        let body = self.parse_body()?;

        return Ok(FunctionNode {
            name: name_token.value.unwrap(),
            args,
            r#type,
            return_var,
            body,
        });
    }

    fn parse_function_arguments(&mut self) -> Result<Vec<ArgumentNode>, SyntaxError> {
        self.expect(Tokens::Punctuation(Punctuations::ParenOpen))?;
        let mut args = Vec::new();

        loop {
            self.advance();
            if matches!(
                self.current().kind,
                Tokens::Punctuation(Punctuations::ParenClose) | Tokens::EOF
            ) {
                break;
            }

            let name_token = self.expect(Tokens::Identifier)?;

            self.advance();
            let mut r#type = None;

            if self.current().kind == Tokens::Punctuation(Punctuations::Colon) {
                self.advance();
                r#type = Some(self.parse_type()?);

                self.advance();
            }

            let mut value = None;

            if self.current().kind
                == Tokens::Punctuation(Punctuations::Assignment(Assignments::Straight))
            {
                self.advance();
                value = Some(self.parse_expression()?);
            }

            if r#type == None && value == None {
                return Err(SyntaxError::default()
                    .ctx(ErrorContext::span(
                        SourcePoint::new(name_token.span.start.ln, name_token.span.start.col),
                        SourcePoint::new(name_token.span.start.ln, name_token.span.end.col),
                        self.src.clone(),
                    ))
                    .kind(SyntaxErrorKind::Expected(SyntaxErrorSource::Token))
                    .msg("argument requires type or default value"));
            }

            args.push(ArgumentNode {
                name: name_token.value.unwrap(),
                r#type,
                default: value,
            });

            if self.current().kind != Tokens::Punctuation(Punctuations::Comma) {
                break;
            }
        }

        self.expect(Tokens::Punctuation(Punctuations::ParenClose))?;
        return Ok(args);
    }
}
