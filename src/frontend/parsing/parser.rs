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
    expressions::{BinaryExpressionKind, ExpressionKind, ExpressionNode},
    items::{ConstantNode, ItemKind, ItemNode},
    namespace::{NamespaceKind, NamespaceNode},
    project::{ProjectKind, ProjectNode},
    r#type::TypeNode,
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

            let result = self.expect(SimplePunctuation(Semicolon));
            if let Ok(_) = result {
                self.project.root.items.push(item);
                self.advance();
            } else {
                self.errors.push(result.unwrap_err());
            }
        }
    }

    fn parse_item(&mut self) -> Result<ItemNode, SyntaxError> {
        let token = self.current();
        let public = token.kind == Keyword(Pub);

        if public {
            self.advance();
        }

        let token = self.current();

        let kind = match token.kind {
            Keyword(Const) => ItemKind::Constant(self.parse_const()?),
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

        return Ok(ConstantNode {
            name: name_token.value.unwrap(),
            r#type,
            value: expression,
        });
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
            let prim_expr = self.parse_primary_expression()?;
            self.advance();

            prim_expr
        };

        return Ok(expr);
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
