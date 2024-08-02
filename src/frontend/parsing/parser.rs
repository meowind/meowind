use std::vec;

use crate::{
    errors::{
        context::ErrorContextBuilder,
        syntax::{SyntaxError, SyntaxErrorKind, SyntaxErrorSource},
        MeowindErrorList,
    },
    frontend::{
        lexing::{
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
    block::{BlockElementKind, BlockElementNode, BlockKind, BlockNode},
    expressions::{BinaryExpressionKind, ExpressionKind, ExpressionNode, ResolutionExpressionKind},
    functions::{ArgumentNode, FunctionNode},
    items::{ConstantNode, ItemKind, ItemNode, StaticNode},
    namespace::{NamespaceKind, NamespaceNode},
    project::{ProjectKind, ProjectNode},
    r#type::TypeNode,
    statements::{StatementKind, StatementNode, VariableDeclarationNode},
};

pub struct Parser<'a> {
    pub project: ProjectNode,
    pub errors: MeowindErrorList<SyntaxError>,

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
        self.expect(ComplexPunctuation(Assignment))?;

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

        self.expect(ComplexPunctuation(Assignment))?;

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

        let body = self.parse_block()?;
        // TODO self.advance();

        return Ok(FunctionNode {
            name: name_token.value.unwrap(),
            args,
            r#type,
            return_var,
            body,
        });
    }

    fn parse_block(&mut self) -> Result<BlockNode, SyntaxError> {
        let token = self.expect_multiple(vec![
            SimplePunctuation(BraceOpen),
            ComplexPunctuation(InlineBlock),
        ])?;

        let block = match token.kind {
            SimplePunctuation(BraceOpen) => self.parse_multiline_block()?,
            ComplexPunctuation(InlineBlock) => self.parse_inline_block()?,
            _ => unreachable!(),
        };

        return Ok(block);
    }

    fn parse_multiline_block(&mut self) -> Result<BlockNode, SyntaxError> {
        self.expect(SimplePunctuation(BraceOpen))?;
        let mut els: Vec<BlockElementNode> = Vec::new();

        loop {
            self.advance();
            let token = self.current();

            if matches!(token.kind, SimplePunctuation(BraceClose) | EOF) {
                break;
            }

            let el = self.parse_block_element()?;
            els.push(el);
        }

        self.expect(SimplePunctuation(BraceClose))?;

        Ok(BlockNode {
            kind: BlockKind::Multiline(els),
        })
    }

    fn parse_inline_block(&mut self) -> Result<BlockNode, SyntaxError> {
        self.expect(ComplexPunctuation(InlineBlock))?;

        self.advance();
        let el = self.parse_block_element()?;

        Ok(BlockNode {
            kind: BlockKind::Inline(Box::new(el)),
        })
    }

    fn parse_block_element(&mut self) -> Result<BlockElementNode, SyntaxError> {
        let token = self.current();

        if token.kind == SimplePunctuation(Semicolon) {
            return Ok(BlockElementNode {
                kind: BlockElementKind::Empty,
            });
        }

        if token.kind == SimplePunctuation(BraceOpen) {
            let block = self.parse_multiline_block()?;

            return Ok(BlockElementNode {
                kind: BlockElementKind::Block(block),
            });
        }

        let stmt = self.parse_statement()?;

        Ok(BlockElementNode {
            kind: BlockElementKind::Statement(stmt),
        })
    }

    fn parse_statement(&mut self) -> Result<StatementNode, SyntaxError> {
        let stmt = match self.current().kind {
            Keyword(Var) => {
                let var = self.parse_variable_declaration()?;

                self.expect(SimplePunctuation(Semicolon))?;

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

        if self.current().kind == ComplexPunctuation(Assignment) {
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

            if self.current().kind == ComplexPunctuation(Assignment) {
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

        self.parse_binary_expression(BinaryExpressionKind::lowest())
    }

    fn parse_binary_expression(
        &mut self,
        bin_kind: BinaryExpressionKind,
    ) -> Result<ExpressionNode, SyntaxError> {
        let mut expr = self.parse_binary_expression_operand(&bin_kind)?;

        while let ComplexPunctuation(punct_kind) = self.current().kind {
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

            /* before
                self.advance();
                prim_expr
            */
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
            errors: MeowindErrorList::new(),
            project: ProjectNode {
                name: "project".to_string(),
                kind: ProjectKind::Program,
                root: NamespaceNode::new(NamespaceKind::Root, Vec::new()),
            },
            src: ScriptSource::default(),
            cursor: 0,
        }
    }
}
