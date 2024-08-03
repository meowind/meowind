use crate::{
    errors::syntax::SyntaxError,
    frontend::{lexing::TokenKind::*, parsing::ast::types::TypeNode},
};

use super::Parser;

impl Parser<'_> {
    pub(super) fn parse_type(&mut self) -> Result<TypeNode, SyntaxError> {
        let type_token = self.expect(Identifier)?;

        Ok(TypeNode {
            raw: type_token.value.unwrap(),
        })
    }
}
