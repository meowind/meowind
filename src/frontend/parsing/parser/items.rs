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
        parsing::ast::items::{ConstantNode, ItemKind, ItemNode, StaticNode},
    },
};

use super::Parser;

impl Parser<'_> {
    pub(super) fn parse_item(&mut self) -> Result<ItemNode, SyntaxError> {
        let mut public = false;
        if self.current().kind == Keyword(Pub) {
            public = true;
            self.advance();
        }

        let token = self.current();
        let kind = match token.kind {
            Keyword(Const) => ItemKind::Constant(self.parse_const()?),
            Keyword(Static) => ItemKind::Static(self.parse_static()?),
            Keyword(Func) => ItemKind::Function(self.parse_function()?),
            _ => {
                return Err(SyntaxError::default()
                    .ctx(
                        ErrorContextBuilder::span(token.loc.start_col, token.loc.end_col)
                            .from_src_and_ln(&self.src, token.loc.ln)
                            .build(),
                    )
                    .kind(SyntaxErrorKind::Unexpected(SyntaxErrorSource::Token)));
            }
        };

        Ok(ItemNode { kind, public })
    }

    fn parse_const(&mut self) -> Result<ConstantNode, SyntaxError> {
        self.expect(Keyword(Const))?;

        self.advance();
        let name_token = self.expect(Identifier)?;

        self.advance();
        self.expect(ComplexPunctuation(Colon))?;

        self.advance();
        let r#type = self.parse_type()?;

        self.advance();
        self.expect(ComplexPunctuation(Assignment(Straight)))?;

        self.advance();
        let expression = self.parse_expression()?;

        self.expect(SimplePunctuation(Semicolon))?;

        return Ok(ConstantNode {
            name: name_token.value.unwrap(),
            r#type,
            value: expression,
        });
    }

    fn parse_static(&mut self) -> Result<StaticNode, SyntaxError> {
        self.expect(Keyword(Static))?;
        self.advance();

        let mut mutable = false;
        if self.current().kind == Keyword(Mut) {
            mutable = true;
            self.advance();
        }

        let name_token = self.expect(Identifier)?;

        self.advance();
        let mut r#type = None;

        if self.current().kind == ComplexPunctuation(Colon) {
            self.advance();
            r#type = Some(self.parse_type()?);

            self.advance();
        }

        self.expect(ComplexPunctuation(Assignment(Straight)))?;

        self.advance();
        let expression = self.parse_expression()?;

        self.expect(SimplePunctuation(Semicolon))?;

        return Ok(StaticNode {
            name: name_token.value.unwrap(),
            r#type,
            value: expression,
            mutable,
        });
    }
}
