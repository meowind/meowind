pub mod bodies;
pub mod expressions;
pub mod functions;
pub mod items;
pub mod statements;
pub mod types;

use crate::{
    errors::{
        context::ErrorContext,
        syntax::{SyntaxError, SyntaxErrorKind, SyntaxErrorSource},
    },
    frontend::lexing::{Token, Tokens},
    source::{SourceFile, SourcePoint},
};

use super::ast::projects::ProjectNode;

pub struct Parser<'a> {
    pub project: ProjectNode,
    pub errors: Vec<SyntaxError>,

    tokens: &'a Vec<Token>,
    src: SourceFile<'a>,
    cursor: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>, src: SourceFile<'a>) -> Parser<'a> {
        Parser {
            tokens,
            src,
            ..Default::default()
        }
    }

    pub fn parse(tokens: &'a Vec<Token>, src: SourceFile<'a>) -> Parser<'a> {
        let mut parser = Parser::new(tokens, src);
        parser.process();

        return parser;
    }

    fn process(&mut self) {
        if self.tokens.is_empty() {
            return;
        }

        while self.current().kind != Tokens::EOF {
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

    fn expect(&self, kind: Tokens) -> Result<Token, SyntaxError> {
        let token = self.current();

        if token.kind != kind {
            let (col, ln) = if let Ok(prev) = self.previous() {
                (prev.span.end.col, prev.span.start.ln)
            } else {
                (token.span.start.col, token.span.start.ln)
            };

            return Err(SyntaxError::default()
                .ctx(ErrorContext::point(ln, col, self.src.clone()))
                .kind(SyntaxErrorKind::Expected(SyntaxErrorSource::Token))
                .msg(format!("expected {}", kind.to_string())));
        };

        Ok(token)
    }

    fn expect_multiple(&self, kinds: Vec<Tokens>) -> Result<Token, SyntaxError> {
        let token = self.current();
        if kinds.contains(&token.kind) {
            return Ok(token);
        }

        let str_kinds: Vec<String> = kinds.iter().map(|kind: &Tokens| kind.to_string()).collect();
        Err(SyntaxError::default()
            .ctx(ErrorContext::span(
                SourcePoint::new(token.span.start.ln, token.span.start.col),
                SourcePoint::new(token.span.start.ln, token.span.end.col),
                self.src.clone(),
            ))
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
            src: SourceFile::default(),
            cursor: 0,
        }
    }
}
