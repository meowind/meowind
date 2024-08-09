use crate::{
    errors::{
        context::ErrorContextBuilder,
        syntax::{SyntaxError, SyntaxErrorKind, SyntaxErrorSource},
    },
    frontend::{
        lexing::{
            AssignmentKind::*, ComplexPunctuationKind::*, KeywordKind::*, SimplePunctuationKind::*,
            TokenKind::*,
        },
        parsing::ast::functions::{ArgumentNode, FunctionNode},
    },
};

use super::Parser;

impl Parser<'_> {
    pub(super) fn parse_function(&mut self) -> Result<FunctionNode, SyntaxError> {
        self.expect(Keyword(Func))?;

        self.advance();
        let name_token = self.expect(Identifier)?;

        self.advance();
        let args = self.parse_function_arguments()?;

        self.advance();
        let mut r#type = None;
        let mut return_var = None;

        if self.current().kind == ComplexPunctuation(ReturnSeparator) {
            self.advance();
            r#type = Some(self.parse_type()?);

            self.advance();
            if self.current().kind == ComplexPunctuation(Colon) {
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
        self.expect(SimplePunctuation(ParenOpen))?;
        let mut args = Vec::new();

        loop {
            self.advance();
            if matches!(self.current().kind, SimplePunctuation(ParenClose) | EOF) {
                break;
            }

            let name_token = self.expect(Identifier)?;

            self.advance();
            let mut r#type = None;

            if self.current().kind == ComplexPunctuation(Colon) {
                self.advance();
                r#type = Some(self.parse_type()?);

                self.advance();
            }

            let mut value = None;

            if self.current().kind == ComplexPunctuation(Assignment(Straight)) {
                self.advance();
                value = Some(self.parse_expression()?);
            }

            if r#type == None && value == None {
                return Err(SyntaxError::default()
                    .ctx(
                        ErrorContextBuilder::span(name_token.loc.start_col, name_token.loc.end_col)
                            .from_src_and_ln(&self.src, name_token.loc.ln)
                            .build(),
                    )
                    .kind(SyntaxErrorKind::Expected(SyntaxErrorSource::Token))
                    .msg("argument requires type or default value"));
            }

            args.push(ArgumentNode {
                name: name_token.value.unwrap(),
                r#type,
                default: value,
            });

            if self.current().kind != SimplePunctuation(Comma) {
                break;
            }
        }

        self.expect(SimplePunctuation(ParenClose))?;
        return Ok(args);
    }
}
