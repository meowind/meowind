pub mod bodies;
pub mod expressions;
pub mod functions;
pub mod items;
pub mod statements;
pub mod types;

use crate::{
    errors::{
        context::ErrorContextBuilder,
        syntax::{SyntaxError, SyntaxErrorKind, SyntaxErrorSource},
    },
    frontend::lexing::{
        Token,
        TokenKind::{self, *},
    },
    structs::ScriptSource,
};

use super::ast::projects::ProjectNode;

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

static DEFAULT_TOKENS: Vec<Token> = Vec::new();

impl<'a> Default for Parser<'a> {
    fn default() -> Self {
        Self {
            tokens: &DEFAULT_TOKENS,
            errors: Vec::new(),
            project: ProjectNode::default(),
            src: ScriptSource::default(),
            cursor: 0,
        }
    }
}
