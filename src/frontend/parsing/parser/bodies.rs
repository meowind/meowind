use crate::{
    errors::syntax::SyntaxError,
    frontend::{
        lexing::{ComplexPunctuationKind::*, SimplePunctuationKind::*, TokenKind::*},
        parsing::ast::bodies::{BodyElementKind, BodyElementNode, BodyKind, BodyNode},
    },
};

use super::Parser;

impl Parser<'_> {
    pub(super) fn parse_body(&mut self) -> Result<BodyNode, SyntaxError> {
        let token = self.expect_multiple(vec![
            SimplePunctuation(BraceOpen),
            ComplexPunctuation(InlineBody),
        ])?;

        let body = match token.kind {
            SimplePunctuation(BraceOpen) => self.parse_multiline_body()?,
            ComplexPunctuation(InlineBody) => self.parse_inline_body()?,
            _ => unreachable!(),
        };

        return Ok(body);
    }

    fn parse_multiline_body(&mut self) -> Result<BodyNode, SyntaxError> {
        self.expect(SimplePunctuation(BraceOpen))?;
        let mut els: Vec<BodyElementNode> = Vec::new();

        loop {
            self.advance();
            let token = self.current();

            if matches!(token.kind, SimplePunctuation(BraceClose) | EOF) {
                break;
            }

            let el = self.parse_body_element()?;
            els.push(el);
        }

        self.expect(SimplePunctuation(BraceClose))?;

        Ok(BodyNode {
            kind: BodyKind::Multiline(els),
        })
    }

    fn parse_inline_body(&mut self) -> Result<BodyNode, SyntaxError> {
        self.expect(ComplexPunctuation(InlineBody))?;

        self.advance();
        let el = self.parse_body_element()?;

        Ok(BodyNode {
            kind: BodyKind::Inline(Box::new(el)),
        })
    }

    fn parse_body_element(&mut self) -> Result<BodyElementNode, SyntaxError> {
        let token = self.current();

        if token.kind == SimplePunctuation(Semicolon) {
            return Ok(BodyElementNode {
                kind: BodyElementKind::Empty,
            });
        }

        if token.kind == SimplePunctuation(BraceOpen) {
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
