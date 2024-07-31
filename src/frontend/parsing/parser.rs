use crate::{
    errors::{
        context::ErrorContextBuilder,
        syntax::{SyntaxError, SyntaxErrorKind, SyntaxErrorSource},
        MeowindErrorList,
    },
    frontend::lexing::{
        ComplexPunctuationKind::{self, *},
        KeywordKind::*,
        LiteralKind::{self, *},
        SimplePunctuationKind::*,
        Token,
        TokenKind::{self, *},
    },
    structs::ScriptSource,
};

use super::ast::{
    expressions::{ExpressionKind, ExpressionNode},
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

            self.project.root.items.push(item);

            self.advance();
            if let Err(err) = self.expect(SimplePunctuation(Semicolon)) {
                self.errors.push(err);
            }
            self.advance();
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
        let type_token = self.expect(Identifier)?;

        self.advance();
        self.expect(ComplexPunctuation(Assignment))?;

        self.advance();
        let expression = self.parse_expression()?;

        return Ok(ConstantNode {
            name: name_token.value.unwrap(),
            r#type: TypeNode {
                raw: type_token.value.unwrap(),
            },
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

        // TODO
        Ok(ExpressionNode {
            kind: ExpressionKind::Literal {
                kind: LiteralKind::Float,
                value: "50".to_string(),
            },
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
