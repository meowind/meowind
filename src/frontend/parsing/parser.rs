use crate::{
    errors::{
        context::ErrorContextBuilder,
        syntax::{SyntaxError, SyntaxErrorKind},
        MeowindErrorList,
    },
    frontend::lexing::{
        ComplexPunctuationKind::{self, *},
        KeywordKind::*,
        LiteralKind::*,
        SimplePunctuationKind::*,
        Token,
        TokenKind::{self, *},
    },
    structs::ScriptSource,
};

use super::ast::{
    items::{ItemKind, ItemNode},
    namespace::{NamespaceKind, NamespaceNode},
    project::{ProjectKind, ProjectNode},
};

use std::string::String as StdString;

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

        self.begin();
    }

    fn current(&self) -> Token {
        self.tokens[self.cursor].clone()
    }

    fn advance(&mut self) -> Token {
        self.cursor += 1;
        self.current()
    }

    fn advance_and_expect(&mut self, kind: TokenKind) {
        let token = self.advance();

        if token.kind != kind {
            self.errors.push(
                SyntaxError::default()
                    .ctx(
                        ErrorContextBuilder::col(token.loc.start_col)
                            .from_src_and_ln(&self.src, token.loc.ln)
                            .build(),
                    )
                    .kind(SyntaxErrorKind::ExpectedToken)
                    .msg(format!("expected {}", kind.to_string())),
            );
        }
    }

    fn begin(&mut self) {
        let token = self.current();

        match token.kind {
            Keyword(Const) => self.parse_const(),
            EOF => {
                return;
            }
            _ => self.errors.push(
                SyntaxError::default()
                    .ctx(
                        ErrorContextBuilder::span(token.loc.start_col, token.loc.end_col)
                            .from_src_and_ln(&self.src, token.loc.ln)
                            .build(),
                    )
                    .kind(SyntaxErrorKind::UnexpectedToken),
            ),
        }
    }

    fn parse_const(&mut self) {}
}

static DEFAULT_TOKENS: &Vec<Token> = &Vec::new();

impl<'a> Default for Parser<'a> {
    fn default() -> Self {
        Self {
            tokens: DEFAULT_TOKENS,
            errors: MeowindErrorList::new(),
            project: ProjectNode::new(
                "project",
                ProjectKind::Program,
                NamespaceNode::new(NamespaceKind::Root, Vec::new()),
            ),
            src: ScriptSource::default(),
            cursor: 0,
        }
    }
}
