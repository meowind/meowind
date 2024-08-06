use crate::{
    errors::{
        context::ErrorContext,
        syntax::{SyntaxError, SyntaxErrorKind, SyntaxErrorSource},
    },
    frontend::{
        lexing::{Assignments, Keywords, Punctuations, Tokens},
        parsing::ast::statements::{
            IfKind, IfNode, StatementKind, StatementNode, VariableDeclarationNode, WhileLoopKind,
            WhileLoopNode,
        },
    },
    source::SourcePoint,
};

use super::Parser;

impl<'a> Parser<'a> {
    pub(super) fn parse_statement(&mut self) -> Result<StatementNode, SyntaxError> {
        let stmt = match self.current().kind {
            Tokens::Keyword(Keywords::Var) => {
                let var = self.parse_variable_declaration()?;
                StatementKind::VariableDeclaration(var)
            }
            Tokens::Keyword(Keywords::Func) => {
                let func = self.parse_function()?;
                StatementKind::FunctionDeclaration(func)
            }
            Tokens::Keyword(Keywords::Return) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Tokens::Punctuation(Punctuations::Semicolon))?;

                StatementKind::Return(expr)
            }
            Tokens::Keyword(Keywords::If) => {
                let if_stmt = self.parse_if_statement()?;
                StatementKind::If(if_stmt)
            }
            Tokens::Keyword(Keywords::While) => {
                let while_loop = self.parse_while_loop()?;
                StatementKind::WhileLoop(while_loop)
            }
            _ => {
                let expr = self.parse_expression()?;
                self.expect(Tokens::Punctuation(Punctuations::Semicolon))?;

                StatementKind::Expression(expr)
            }
        };

        return Ok(StatementNode { kind: stmt });
    }

    fn parse_variable_declaration(&mut self) -> Result<VariableDeclarationNode, SyntaxError> {
        self.expect(Tokens::Keyword(Keywords::Var))?;
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
                .msg("variable requires type or default value"));
        }

        self.expect(Tokens::Punctuation(Punctuations::Semicolon))?;

        return Ok(VariableDeclarationNode {
            name: name_token.value.unwrap(),
            r#type,
            value,
            mutable,
        });
    }

    fn parse_if_statement(&mut self) -> Result<IfNode, SyntaxError> {
        self.expect(Tokens::Keyword(Keywords::If))?;
        self.advance();

        let cond = self.parse_expression()?;
        let body = self.parse_body()?;

        self.advance();

        let mut r#else = None;
        if self.current().kind == Tokens::Keyword(Keywords::Else) {
            self.advance();

            if self.current().kind == Tokens::Keyword(Keywords::If) {
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
        self.expect(Tokens::Keyword(Keywords::While))?;
        self.advance();

        let cond = self.parse_expression()?;
        let body = self.parse_body()?;

        self.advance();

        let mut r#else = None;
        if self.current().kind == Tokens::Keyword(Keywords::Else) {
            self.advance();

            if self.current().kind == Tokens::Keyword(Keywords::While) {
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
