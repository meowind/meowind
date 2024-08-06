use crate::{
    errors::{
        context::ErrorContext,
        syntax::{SyntaxError, SyntaxErrorKind, SyntaxErrorSource},
    },
    frontend::{
        lexing::{Assignments, Keywords, Punctuations, Tokens},
        parsing::ast::items::{ConstantNode, ItemKind, ItemNode, StaticNode},
    },
    source::SourcePoint,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_item(&mut self) -> Result<ItemNode, SyntaxError> {
        let mut public = false;
        if self.current().kind == Tokens::Keyword(Keywords::Pub) {
            public = true;
            self.advance();
        }

        let token = self.current();
        let kind = match token.kind {
            Tokens::Keyword(Keywords::Const) => ItemKind::Constant(self.parse_const()?),
            Tokens::Keyword(Keywords::Static) => ItemKind::Static(self.parse_static()?),
            Tokens::Keyword(Keywords::Func) => ItemKind::Function(self.parse_function()?),
            _ => {
                return Err(SyntaxError::default()
                    .ctx(ErrorContext::span(
                        SourcePoint::new(token.span.start.ln, token.span.start.col),
                        SourcePoint::new(token.span.start.ln, token.span.end.col),
                        self.src.clone(),
                    ))
                    .kind(SyntaxErrorKind::Unexpected(SyntaxErrorSource::Token)));
            }
        };

        Ok(ItemNode { kind, public })
    }

    fn parse_const(&mut self) -> Result<ConstantNode, SyntaxError> {
        self.expect(Tokens::Keyword(Keywords::Const))?;

        self.advance();
        let name_token = self.expect(Tokens::Identifier)?;

        self.advance();
        self.expect(Tokens::Punctuation(Punctuations::Colon))?;

        self.advance();
        let r#type = self.parse_type()?;

        self.advance();
        self.expect(Tokens::Punctuation(Punctuations::Assignment(
            Assignments::Straight,
        )))?;

        self.advance();
        let expression = self.parse_expression()?;

        self.expect(Tokens::Punctuation(Punctuations::Semicolon))?;

        return Ok(ConstantNode {
            name: name_token.value.unwrap(),
            r#type,
            value: expression,
        });
    }

    fn parse_static(&mut self) -> Result<StaticNode, SyntaxError> {
        self.expect(Tokens::Keyword(Keywords::Static))?;
        self.advance();

        let mut mutable = false;
        if self.current().kind == Tokens::Keyword(Keywords::Mut) {
            mutable = true;
            self.advance();
        }

        let name_token = self.expect(Tokens::Identifier)?;

        self.advance();
        let mut r#type = None;

        if self.current().kind == Tokens::Punctuation(Punctuations::Colon) {
            self.advance();
            r#type = Some(self.parse_type()?);

            self.advance();
        }

        self.expect(Tokens::Punctuation(Punctuations::Assignment(
            Assignments::Straight,
        )))?;

        self.advance();
        let expression = self.parse_expression()?;

        self.expect(Tokens::Punctuation(Punctuations::Semicolon))?;

        return Ok(StaticNode {
            name: name_token.value.unwrap(),
            r#type,
            value: expression,
            mutable,
        });
    }
}
