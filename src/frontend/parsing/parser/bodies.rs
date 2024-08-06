use crate::{
    errors::syntax::SyntaxError,
    frontend::{
        lexing::{Punctuations, Tokens},
        parsing::ast::bodies::{BodyElementKind, BodyElementNode, BodyKind, BodyNode},
    },
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_body(&mut self) -> Result<BodyNode, SyntaxError> {
        let token = self.expect_multiple(vec![
            Tokens::Punctuation(Punctuations::BraceOpen),
            Tokens::Punctuation(Punctuations::InlineBody),
        ])?;

        let body = match token.kind {
            Tokens::Punctuation(Punctuations::BraceOpen) => self.parse_multiline_body()?,
            Tokens::Punctuation(Punctuations::InlineBody) => self.parse_inline_body()?,
            _ => unreachable!(),
        };

        return Ok(body);
    }

    fn parse_multiline_body(&mut self) -> Result<BodyNode, SyntaxError> {
        self.expect(Tokens::Punctuation(Punctuations::BraceOpen))?;
        let mut els: Vec<BodyElementNode> = Vec::new();

        loop {
            self.advance();
            let token = self.current();

            if matches!(
                token.kind,
                Tokens::Punctuation(Punctuations::BraceClose) | Tokens::EOF
            ) {
                break;
            }

            let el = self.parse_body_element()?;
            els.push(el);
        }

        self.expect(Tokens::Punctuation(Punctuations::BraceClose))?;

        Ok(BodyNode {
            kind: BodyKind::Multiline(els),
        })
    }

    fn parse_inline_body(&mut self) -> Result<BodyNode, SyntaxError> {
        self.expect(Tokens::Punctuation(Punctuations::InlineBody))?;

        self.advance();
        let el = self.parse_body_element()?;

        Ok(BodyNode {
            kind: BodyKind::Inline(Box::new(el)),
        })
    }

    fn parse_body_element(&mut self) -> Result<BodyElementNode, SyntaxError> {
        let token = self.current();

        if token.kind == Tokens::Punctuation(Punctuations::Semicolon) {
            return Ok(BodyElementNode {
                kind: BodyElementKind::Empty,
            });
        }

        if token.kind == Tokens::Punctuation(Punctuations::BraceOpen) {
            let body = self.parse_multiline_body()?;

            return Ok(BodyElementNode {
                kind: BodyElementKind::Body(body),
            });
        }

        let stmt = self.parse_statement()?;

        Ok(BodyElementNode {
            kind: BodyElementKind::Statement(stmt),
        })
    }
}
