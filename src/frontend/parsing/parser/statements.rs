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
        parsing::ast::statements::{
            IfKind, IfNode, StatementKind, StatementNode, VariableDeclarationNode, WhileLoopKind,
            WhileLoopNode,
        },
    },
};

use super::Parser;

impl Parser<'_> {
    pub(super) fn parse_statement(&mut self) -> Result<StatementNode, SyntaxError> {
        let stmt = match self.current().kind {
            Keyword(Var) => {
                let var = self.parse_variable_declaration()?;
                StatementKind::VariableDeclaration(var)
            }
            Keyword(Func) => {
                let func = self.parse_function()?;
                StatementKind::FunctionDeclaration(func)
            }
            Keyword(Return) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(SimplePunctuation(Semicolon))?;

                StatementKind::Return(expr)
            }
            Keyword(If) => {
                let if_stmt = self.parse_if_statement()?;
                StatementKind::If(if_stmt)
            }
            Keyword(While) => {
                let while_loop = self.parse_while_loop()?;
                StatementKind::WhileLoop(while_loop)
            }
            _ => {
                let expr = self.parse_expression()?;
                self.expect(SimplePunctuation(Semicolon))?;

                StatementKind::Expression(expr)
            }
        };

        return Ok(StatementNode { kind: stmt });
    }

    fn parse_variable_declaration(&mut self) -> Result<VariableDeclarationNode, SyntaxError> {
        self.expect(Keyword(Var))?;
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
                .msg("variable requires type or default value"));
        }

        self.expect(SimplePunctuation(Semicolon))?;

        return Ok(VariableDeclarationNode {
            name: name_token.value.unwrap(),
            r#type,
            value,
            mutable,
        });
    }

    fn parse_if_statement(&mut self) -> Result<IfNode, SyntaxError> {
        self.expect(Keyword(If))?;
        self.advance();

        let cond = self.parse_expression()?;
        let body = self.parse_body()?;

        self.advance();

        let mut r#else = None;
        if self.current().kind == Keyword(Else) {
            self.advance();

            if self.current().kind == Keyword(If) {
                let else_if = self.parse_if_statement()?;
                r#else = Some(Box::new(else_if));
            } else {
                let else_body = self.parse_body()?;
                r#else = Some(Box::new(IfNode {
                    kind: IfKind::Else,
                    body: else_body,
                }));
            }
        }

        return Ok(IfNode {
            kind: IfKind::If { cond, r#else },
            body,
        });
    }

    fn parse_while_loop(&mut self) -> Result<WhileLoopNode, SyntaxError> {
        self.expect(Keyword(While))?;
        self.advance();

        let cond = self.parse_expression()?;
        let body = self.parse_body()?;

        self.advance();

        let mut r#else = None;
        if self.current().kind == Keyword(Else) {
            self.advance();

            if self.current().kind == Keyword(While) {
                let else_while = self.parse_while_loop()?;
                r#else = Some(Box::new(else_while));
            } else {
                let else_body = self.parse_body()?;
                r#else = Some(Box::new(WhileLoopNode {
                    kind: WhileLoopKind::Else,
                    body: else_body,
                }));
            }
        }

        return Ok(WhileLoopNode {
            kind: WhileLoopKind::While { cond, r#else },
            body,
        });
    }
}
