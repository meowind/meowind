use crate::{
    errors::{syntax::SyntaxError, MeowindErrorList},
    frontend::lexing::Token,
};

use super::ast::{
    namespace::{NamespaceKind, NamespaceNode},
    project::{ProjectKind, ProjectNode},
};

pub struct Parser<'a> {
    pub project: ProjectNode,
    pub errors: MeowindErrorList<SyntaxError>,

    tokens: &'a Vec<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Parser {
        Parser {
            tokens,
            ..Default::default()
        }
    }

    pub fn parse(tokens: &'a Vec<Token>) -> Parser<'a> {
        let mut parser = Parser::new(tokens);
        parser.process();

        return parser;
    }

    fn process(&mut self) {}
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
        }
    }
}
