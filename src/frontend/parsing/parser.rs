use std::vec;

use crate::{
    errors::{
        context::ErrorContextBuilder,
        syntax::{SyntaxError, SyntaxErrorKind, SyntaxErrorSource},
    },
    frontend::{
        lexing::{
            AssignmentKind::*,
            ComplexPunctuationKind::*,
            KeywordKind::*,
            SimplePunctuationKind::*,
            Token,
            TokenKind::{self, *},
        },
        parsing::ast::expressions::UnaryExpressionKind,
    },
    structs::ScriptSource,
};

use super::ast::{
    body::{BodyElementKind, BodyElementNode, BodyKind, BodyNode},
    expressions::{BinaryExpressionKind, ExpressionKind, ExpressionNode, ResolutionExpressionKind},
    functions::{ArgumentNode, FunctionNode},
    items::{ConstantNode, ItemKind, ItemNode, StaticNode},
    project::ProjectNode,
    r#type::TypeNode,
    statements::{
        IfKind, IfNode, StatementKind, StatementNode, VariableDeclarationNode, WhileLoopKind,
        WhileLoopNode,
    },
};

pub struct Parser<'a> {
    pub project: ProjectNode,
    pub errors: Vec<SyntaxError>,

    tokens: &'a Vec<Token>,
    src: ScriptSource<'a>,
    cursor: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>, src: ScriptSource<'a>) -> Parser<'a> {
        Parser {
            tokens,
            src,
            ..Default::default()
        }
    }

    pub fn parse(tokens: &'a Vec<Token>, src: ScriptSource<'a>) -> Parser<'a> {
        let mut parser = Parser::new(tokens, src);
        parser.process();

        return parser;
    }

    fn process(&mut self) {
        if self.tokens.is_empty() {
            return;
        }

        while self.current().kind != EOF {
            let result = self.parse_item();
            let Ok(item) = result else {
                self.errors.push(result.unwrap_err());
                return;
            };

            self.project.root.items.push(item);
            self.advance();
        }
    }

    fn parse_item(&mut self) -> Result<ItemNode, SyntaxError> {
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

    fn parse_function(&mut self) -> Result<FunctionNode, SyntaxError> {
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

    fn parse_body(&mut self) -> Result<BodyNode, SyntaxError> {
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

    fn parse_statement(&mut self) -> Result<StatementNode, SyntaxError> {
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

    fn parse_expression(&mut self) -> Result<ExpressionNode, SyntaxError> {
        let token = self.current();
        if token.kind == EOF {
            return Err(SyntaxError::default()
                .ctx(
                    ErrorContextBuilder::col(token.loc.start_col)
                        .from_src_and_ln(&self.src, token.loc.ln)
                        .build(),
                )
                .kind(SyntaxErrorKind::Expected(SyntaxErrorSource::Expression)));
        }

        self.parse_assignment_expression()
    }

    fn parse_assignment_expression(&mut self) -> Result<ExpressionNode, SyntaxError> {
        let mut left = self.parse_binary_expression(BinaryExpressionKind::lowest())?;

        if let ComplexPunctuation(Assignment(kind)) = self.current().kind {
            self.advance();
            let right = self.parse_assignment_expression()?;

            left = ExpressionNode {
                kind: ExpressionKind::Assignment {
                    left: Box::new(left),
                    op: kind,
                    right: Box::new(right),
                },
            };
        }

        return Ok(left);
    }

    fn parse_binary_expression(
        &mut self,
        bin_kind: BinaryExpressionKind,
    ) -> Result<ExpressionNode, SyntaxError> {
        let mut expr = self.parse_binary_expression_operand(&bin_kind)?;

        while let ComplexPunctuation(punct_kind) = self.current().kind {
            if matches!(punct_kind, Assignment(_) | InlineBody) {
                break;
            }

            let token = self.current();

            let Ok(punct_bin_kind) = BinaryExpressionKind::from_punct(&punct_kind) else {
                return Err(SyntaxError::default()
                    .ctx(
                        ErrorContextBuilder::span(token.loc.start_col, token.loc.end_col)
                            .from_src_and_ln(&self.src, token.loc.ln)
                            .build(),
                    )
                    .kind(SyntaxErrorKind::Unexpected(SyntaxErrorSource::Token))
                    .msg("specified token is not a binary operator"));
            };

            if punct_bin_kind != bin_kind {
                break;
            }

            self.advance();

            let right = self.parse_binary_expression_operand(&bin_kind)?;
            expr = ExpressionNode {
                kind: ExpressionKind::Binary {
                    kind: punct_bin_kind.clone(),
                    left: Box::new(expr),
                    op: punct_kind,
                    right: Box::new(right),
                },
            }
        }

        return Ok(expr);
    }

    fn parse_binary_expression_operand(
        &mut self,
        bin_kind: &BinaryExpressionKind,
    ) -> Result<ExpressionNode, SyntaxError> {
        let expr = if let Ok(more_precedence) =
            BinaryExpressionKind::from_precedence(bin_kind.precedence() + 1)
        {
            self.parse_binary_expression(more_precedence)?
        } else {
            self.parse_call_or_resolution_expression()?
        };

        return Ok(expr);
    }

    fn parse_call_or_resolution_expression(&mut self) -> Result<ExpressionNode, SyntaxError> {
        let res = self.parse_resolution_expression()?;

        if self.current().kind == SimplePunctuation(ParenOpen) {
            return Ok(self.parse_call_expression(res)?);
        }

        return Ok(res);
    }

    fn parse_call_expression(
        &mut self,
        res: ExpressionNode,
    ) -> Result<ExpressionNode, SyntaxError> {
        let args = self.parse_call_arguments()?;

        let mut expr = ExpressionNode {
            kind: ExpressionKind::Call {
                res: Box::new(res),
                args,
            },
        };

        self.advance();
        if self.current().kind == SimplePunctuation(ParenOpen) {
            expr = self.parse_call_expression(expr)?;
        }

        return Ok(expr);
    }

    fn parse_call_arguments(&mut self) -> Result<Vec<ExpressionNode>, SyntaxError> {
        self.expect(SimplePunctuation(ParenOpen))?;
        let mut args = Vec::new();

        loop {
            self.advance();
            if matches!(self.current().kind, SimplePunctuation(ParenClose) | EOF) {
                break;
            }

            let expr = self.parse_expression()?;
            args.push(expr);

            if self.current().kind != SimplePunctuation(Comma) {
                break;
            }
        }

        self.expect(SimplePunctuation(ParenClose))?;
        return Ok(args);
    }

    fn parse_resolution_expression(&mut self) -> Result<ExpressionNode, SyntaxError> {
        let mut left = self.parse_primary_expression()?;

        loop {
            self.advance();

            if !matches!(
                self.current().kind,
                ComplexPunctuation(MemberSeparator) | ComplexPunctuation(NamespaceSeparator)
            ) {
                break;
            }

            let kind = match self.current().kind {
                ComplexPunctuation(MemberSeparator) => ResolutionExpressionKind::Member,
                ComplexPunctuation(NamespaceSeparator) => ResolutionExpressionKind::Namespace,
                _ => unreachable!(),
            };

            self.advance();
            let right = self.parse_primary_expression()?;

            left = ExpressionNode {
                kind: ExpressionKind::Resolution {
                    left: Box::new(left),
                    right: Box::new(right),
                    kind,
                },
            };
        }

        return Ok(left);
    }

    fn parse_primary_expression(&mut self) -> Result<ExpressionNode, SyntaxError> {
        let token = self.current();

        match token.kind {
            Identifier => Ok(ExpressionNode {
                kind: ExpressionKind::Identifier {
                    name: token.value.unwrap(),
                },
            }),
            Literal(lit) => Ok(ExpressionNode {
                kind: ExpressionKind::Literal {
                    kind: lit,
                    value: token.value.unwrap(),
                },
            }),

            ComplexPunctuation(punct_kind) => {
                let Ok(un_kind) = UnaryExpressionKind::from_punct(&punct_kind) else {
                    return Err(SyntaxError::default()
                        .ctx(
                            ErrorContextBuilder::span(token.loc.start_col, token.loc.end_col)
                                .from_src_and_ln(&self.src, token.loc.ln)
                                .build(),
                        )
                        .kind(SyntaxErrorKind::Unexpected(SyntaxErrorSource::Token))
                        .msg("specified token is not a unary operator"));
                };

                self.advance();
                let right = self.parse_primary_expression()?;

                Ok(ExpressionNode {
                    kind: ExpressionKind::Unary {
                        kind: un_kind,
                        op: punct_kind,
                        right: Box::new(right),
                    },
                })
            }
            SimplePunctuation(ParenOpen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(SimplePunctuation(ParenClose))?;

                Ok(expr)
            }

            _ => Err(SyntaxError::default()
                .ctx(
                    ErrorContextBuilder::span(token.loc.start_col, token.loc.end_col)
                        .from_src_and_ln(&self.src, token.loc.ln)
                        .build(),
                )
                .kind(SyntaxErrorKind::Unexpected(SyntaxErrorSource::Token))
                .msg("specified token cannot be used for expressions")),
        }
    }

    fn parse_type(&mut self) -> Result<TypeNode, SyntaxError> {
        let type_token = self.expect(Identifier)?;

        Ok(TypeNode {
            raw: type_token.value.unwrap(),
        })
    }

    fn previous(&self) -> Result<Token, ()> {
        if self.cursor == 0 {
            return Err(());
        }

        Ok(self.tokens[self.cursor - 1].clone())
    }

    fn current(&self) -> Token {
        self.tokens[self.cursor].clone()
    }

    fn advance(&mut self) {
        self.cursor += 1;
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, SyntaxError> {
        let token = self.current();

        if token.kind != kind {
            let (col, ln) = if let Ok(prev) = self.previous() {
                (prev.loc.end_col, prev.loc.ln)
            } else {
                (token.loc.start_col, token.loc.ln)
            };

            return Err(SyntaxError::default()
                .ctx(
                    ErrorContextBuilder::col(col)
                        .from_src_and_ln(&self.src, ln)
                        .build(),
                )
                .kind(SyntaxErrorKind::Expected(SyntaxErrorSource::Token))
                .msg(format!("expected {}", kind.to_string())));
        };

        Ok(token)
    }

    fn expect_multiple(&mut self, kinds: Vec<TokenKind>) -> Result<Token, SyntaxError> {
        let token = self.current();
        if kinds.contains(&token.kind) {
            return Ok(token);
        }

        let str_kinds: Vec<String> = kinds.iter().map(|kind| kind.to_string()).collect();
        Err(SyntaxError::default()
            .ctx(
                ErrorContextBuilder::span(self.current().loc.start_col, self.current().loc.end_col)
                    .from_src_and_ln(&self.src, self.current().loc.ln)
                    .build(),
            )
            .kind(SyntaxErrorKind::Expected(SyntaxErrorSource::Token))
            .msg(format!("expected {}", str_kinds.join(" or "))))
    }
}

static DEFAULT_TOKENS: &Vec<Token> = &Vec::new();

impl<'a> Default for Parser<'a> {
    fn default() -> Self {
        Self {
            tokens: DEFAULT_TOKENS,
            errors: Vec::new(),
            project: ProjectNode::default(),
            src: ScriptSource::default(),
            cursor: 0,
        }
    }
}
