// TODO: undefined errors

#[cfg(debug_assertions)]
use crate::utils::Stopwatch;

use super::tokens::{
    ComplexPunctuationKind::{self, *},
    KeywordKind,
    LiteralKind::*,
    SimplePunctuationKind, Token,
    TokenKind::{self, *},
    Tokens,
};
use crate::{
    debug,
    errors::{
        syntax::{SyntaxError, SyntaxErrorKind},
        MeowindErrorList,
    },
    structs::MeowindScriptSource,
};
use std::{fmt, str::FromStr};
use unicode_segmentation::UnicodeSegmentation;

pub struct Lexer<'a> {
    pub src: MeowindScriptSource<'a>,

    tokens: Tokens,
    errors: MeowindErrorList<SyntaxError>,

    ln: usize,
    col: usize,
    kind_buf: TokenKind,
    value_buf: LexerValueBuffer,
    punct_buf: LexerValueBuffer,
}

impl<'a> Lexer<'a> {
    pub fn new(source: MeowindScriptSource) -> Lexer {
        Lexer {
            src: source,
            tokens: Tokens::new(),
            errors: MeowindErrorList::new(),

            ln: 1,
            col: 0,
            kind_buf: Undefined,
            value_buf: LexerValueBuffer::new(),
            punct_buf: LexerValueBuffer::new(),
        }
    }

    pub fn tokenize(&mut self) -> (&Tokens, &MeowindErrorList<SyntaxError>) {
        debug!("== LEXER AWAKE ==\n");

        #[cfg(debug_assertions)]
        let mut stopwatch = Stopwatch::start_new();

        self.tokens.vector.clear();
        self.errors.errors.clear();
        self.reset_buffers();

        for ch in self.src.contents.clone().chars() {
            if ch == '\n' {
                self.ln += 1;
                self.col = 0;

                self.process_keyword();
                continue;
            }
            self.col += 1;

            if let Ok(kind) = SimplePunctuationKind::from_char(ch) {
                debug!(
                    "found simple punctuation, pushing \"{}\" and then \"{ch}\"\n",
                    self.value_buf
                );

                if !self.punct_buf.is_empty() {
                    self.process_complex_punctuation(ch);
                } else {
                    self.tokens.push_not_empty(Token::new(
                        self.ln,
                        self.col - self.value_buf.count(),
                        self.kind_buf.clone(),
                        Some(self.value_buf.value.clone()),
                    ));
                }
                self.reset_buffers();

                self.tokens
                    .push_new(self.ln, self.col, SimplePunctuation(kind), None);
                continue;
            }

            if ch.is_ascii_punctuation() && ch != '_' {
                debug!("found complex punctuation: {ch}\n");

                self.punct_buf.push(ch);
                continue;
            }

            if !self.punct_buf.is_empty() {
                self.process_complex_punctuation(ch);
            }

            if ch.is_whitespace() {
                self.process_keyword();
                continue;
            }

            match &self.kind_buf {
                Undefined => {
                    if ch.is_alphabetic() || ch == '_' {
                        self.kind_buf = Identifier;
                    } else if ch.is_digit(10) {
                        self.kind_buf = Literal(Integer);
                    }
                }
                Literal(lit) if lit.is_number() => {
                    if ch.is_alphabetic() && ch != 'E' && ch != 'e' {
                        self.kind_buf = InvalidIdentifier;

                        self.errors.push(SyntaxError::new_with_context(
                            SyntaxErrorKind::UnexpectedCharacter,
                            "identifiers cannot start with a digit",
                            self.ln,
                            self.current_line(),
                            self.col - self.value_buf.count(),
                            self.col,
                            self.src.path.clone(),
                        ));
                    }
                }
                _ => {}
            }

            self.value_buf.push(ch);

            debug!(
                "pushed {ch} to buffer\ncurrent value: {}\ncurrent kind: {:?}\nlocation: ({}, {})\n",
                self.value_buf,
                self.kind_buf,
                self.ln,
                self.col
            );
        }

        self.tokens.push_new(self.ln, self.col + 1, EOF, None);

        debug!(
            "== LEXER FINISHED ==\nelapsed time: {}μs = {}ms\ntotal tokens: {}\n",
            stopwatch.micros(),
            stopwatch.millis(),
            self.tokens.vector.len()
        );

        return (&self.tokens, &self.errors);
    }

    fn current_line(&self) -> String {
        self.src.lines[self.ln - 1].to_string()
    }

    fn process_keyword(&mut self) {
        if let Ok(kind) = KeywordKind::from_str(&self.value_buf.value) {
            self.tokens.push_new(
                self.ln,
                self.col - self.value_buf.count(),
                Keyword(kind),
                None,
            );
        } else {
            self.tokens.push_new_not_empty(
                self.ln,
                self.col - self.value_buf.count(),
                self.kind_buf.clone(),
                self.value_buf.value.clone(),
            );
        }
        self.reset_buffers();
    }

    fn process_complex_punctuation(&mut self, ch: char) {
        match self.punct_buf.value.as_str() {
            "." => {
                self.recognize_dot(ch);
            }
            "-" => {
                self.recognize_minus(ch);
            }
            _ => {
                self.tokens.push_new_not_empty(
                    self.ln,
                    self.col - self.value_buf.count() - self.punct_buf.count(),
                    self.kind_buf.clone(),
                    self.value_buf.value.clone(),
                );
                self.reset_buffers();

                debug!("starting decomposing \"{ch}\" to multiple tokens\n");

                self.decompose_complex_punctuation();
            }
        }

        self.punct_buf = LexerValueBuffer::new();
    }

    fn recognize_dot(&mut self, ch: char) {
        if ch.is_digit(10) {
            debug!("recognized \".\" as a part of float\n");

            self.kind_buf = Literal(Float);
            self.value_buf.push('.');
        } else {
            debug!("recognized \".\" as a member separator\n");

            self.tokens.push_new_not_empty(
                self.ln,
                self.col - self.value_buf.count(),
                self.kind_buf.clone(),
                self.value_buf.value.clone(),
            );
            self.reset_buffers();

            self.tokens.push_new(
                self.ln,
                self.col - 1,
                ComplexPunctuation(MemberSeparator),
                None,
            );
        }
    }

    fn recognize_minus(&mut self, ch: char) {
        if ch.is_digit(10)
            && (self.value_buf.value.ends_with("e") || self.value_buf.value.ends_with("E"))
            && let Literal(lit) = &self.kind_buf
            && lit.is_number()
        {
            debug!("recognized \"-\" as a part of E notation\n");

            self.kind_buf = Literal(Float);
            self.value_buf.push('-');
        } else {
            self.tokens.push_new_not_empty(
                self.ln,
                self.col - self.value_buf.count() - 1,
                self.kind_buf.clone(),
                self.value_buf.value.clone(),
            );
            self.reset_buffers();

            debug!("recognized \"-\" as a minus operator\n");

            self.tokens.push_new(
                self.ln,
                self.col - 1,
                ComplexPunctuation(OperatorMinus),
                None,
            );
        }
    }

    fn decompose_complex_punctuation(&mut self) {
        if self.punct_buf.count() == 1 {
            if let Ok(kind) = ComplexPunctuationKind::from_str(&self.punct_buf.value) {
                self.tokens
                    .push_new(self.ln, self.col - 1, ComplexPunctuation(kind), None);

                return;
            } else {
                self.tokens.push_new(
                    self.ln,
                    self.col - 1,
                    Undefined,
                    Some(self.punct_buf.value.clone()),
                );

                return;
            }
        }

        let mut punct_char_idx = 0;
        while punct_char_idx < self.punct_buf.count() {
            let mut current_punct_buf = String::new();
            let mut valid_punct_kind = Undefined;

            let from_char_idx = punct_char_idx;
            for pi in punct_char_idx..self.punct_buf.count() {
                let pc = self.punct_buf.value.clone().chars().nth(pi).unwrap();
                current_punct_buf.push(pc);

                if let Ok(kind) = ComplexPunctuationKind::from_str(&current_punct_buf) {
                    punct_char_idx = pi + 1;
                    valid_punct_kind = ComplexPunctuation(kind);
                }
            }

            if valid_punct_kind == Undefined {
                self.tokens.push_new(
                    self.ln,
                    self.col - self.punct_buf.count() + from_char_idx,
                    valid_punct_kind.clone(),
                    Some(current_punct_buf),
                );

                break;
            }

            self.tokens.push_new(
                self.ln,
                self.col - self.punct_buf.count() + from_char_idx,
                valid_punct_kind.clone(),
                None,
            );
        }
    }

    fn reset_buffers(&mut self) {
        self.value_buf = LexerValueBuffer::new();
        self.kind_buf = Undefined;
    }
}

pub struct LexerValueBuffer {
    value: String,
}

impl LexerValueBuffer {
    pub fn new() -> LexerValueBuffer {
        LexerValueBuffer {
            value: String::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.value.graphemes(true).count()
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn push(&mut self, c: char) {
        self.value.push(c);
    }
}

impl fmt::Display for LexerValueBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}
