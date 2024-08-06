use crate::{
    errors::{
        context::ErrorContext,
        syntax::{SyntaxError, SyntaxErrorKind, SyntaxErrorSource},
    },
    source::{SourceFile, SourcePoint, SourceSpan, DEFAULT_SRC_CONTENTS},
    utils::string::StringUtils,
};
use std::{path::PathBuf, str::FromStr};

use super::{Keywords, Literals, Punctuations, Token, Tokens};

pub struct Lexer<'a> {
    pub src: SourceFile<'a>,

    pub tokens: Vec<Token>,
    pub errors: Vec<SyntaxError<'a>>,

    cur_ln: usize,
    cur_col: usize,
    start_col_buf: usize,
    kind_buf: Tokens,
    value_buf: String,
    punct_buf: String,

    inside_string: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: SourceFile) -> Lexer {
        Lexer {
            src: source,

            ..Default::default()
        }
    }

    pub fn tokenize(source: SourceFile<'a>) -> Lexer<'a> {
        let mut lexer = Lexer::new(source);
        lexer.process();

        return lexer;
    }

    fn process(&mut self) {
        if self.src.contents.is_empty() {
            return;
        }

        for ch in self.src.contents.chars() {
            self.iteration(ch);
        }
        self.cur_col += 1;

        if self.inside_string {
            self.errors.push(
                SyntaxError::default()
                    .ctx(ErrorContext::point(
                        self.cur_ln,
                        self.cur_col,
                        self.src.clone(),
                    ))
                    .kind(SyntaxErrorKind::Expected(SyntaxErrorSource::Character))
                    .msg("expected double quote to close string literal"),
            );
        }

        if !self.punct_buf.is_empty() {
            self.process_complex_punctuation('\n');
        } else {
            self.push_keyword_or_ident(SourceSpan::one_ln(
                self.cur_ln,
                self.start_col_buf,
                self.cur_col,
            ));
        }

        self.push_new(
            SourceSpan::one_ln(self.cur_ln, self.cur_col, self.cur_col),
            Tokens::EOF,
            None,
        );
    }

    fn iteration(&mut self, ch: char) {
        // TODO: comments
        // TODO: strings and escape sequences
        // TODO: interpolated strings

        if ch == '\r' {
            return;
        }

        if ch == '\n' {
            self.cur_col += 1;

            if self.inside_string {
                self.errors.push(
                    SyntaxError::default()
                        .ctx(ErrorContext::point(
                            self.cur_ln,
                            self.cur_col,
                            self.src.clone(),
                        ))
                        .kind(SyntaxErrorKind::Expected(SyntaxErrorSource::Character))
                        .msg("regular string literals cannot be over multiple lines"),
                );

                self.inside_string = false;
                self.reset_buffers();
            } else {
                if !self.punct_buf.is_empty() {
                    self.process_complex_punctuation('\n');
                } else {
                    self.push_keyword_or_ident(SourceSpan::one_ln(
                        self.cur_ln,
                        self.start_col_buf,
                        self.cur_col,
                    ));
                }
            }

            self.cur_ln += 1;
            self.cur_col = 0;
            self.start_col_buf = 1;

            return;
        }

        self.cur_col += 1;

        if ch == '"' {
            self.inside_string = !self.inside_string;

            if self.inside_string {
                if !self.punct_buf.is_empty() {
                    self.process_complex_punctuation(ch);
                } else {
                    self.push_keyword_or_ident(SourceSpan::one_ln(
                        self.cur_ln,
                        self.start_col_buf,
                        self.cur_col,
                    ));
                }

                self.start_col_buf = self.cur_col;
                self.kind_buf = Tokens::Literal(Literals::String);
            } else {
                self.push_new(
                    SourceSpan::one_ln(self.cur_ln, self.start_col_buf, self.cur_col + 1),
                    Tokens::Literal(Literals::String),
                    Some(self.value_buf.clone()),
                );
                self.reset_buffers();

                self.start_col_buf = self.cur_col + 1;
                self.kind_buf = Tokens::Undefined;
            }

            return;
        }

        if self.inside_string {
            self.value_buf.push(ch);
            return;
        }

        if ch.is_ascii_punctuation() && ch != '_' {
            if self.punct_buf.is_empty() && self.kind_buf != Tokens::Literal(Literals::Integer) {
                self.push_keyword_or_ident(SourceSpan::one_ln(
                    self.cur_ln,
                    self.start_col_buf,
                    self.cur_col,
                ));
                self.start_col_buf = self.cur_col;
            }

            self.punct_buf.push(ch);
            return;
        }

        if !self.punct_buf.is_empty() {
            self.process_complex_punctuation(ch);
        }

        if ch.is_whitespace() {
            self.push_keyword_or_ident(SourceSpan::one_ln(
                self.cur_ln,
                self.start_col_buf,
                self.cur_col,
            ));

            self.start_col_buf = self.cur_col + 1;
            return;
        }

        match &self.kind_buf {
            Tokens::Undefined => {
                if ch.is_alphabetic() || ch == '_' {
                    self.kind_buf = Tokens::Identifier;
                } else if ch.is_digit(10) {
                    self.kind_buf = Tokens::Literal(Literals::Integer);
                }
            }
            Tokens::Literal(lit) if lit.is_number() => {
                if ch.is_alphabetic() && ch != 'E' && ch != 'e' {
                    self.kind_buf = Tokens::InvalidIdentifier;

                    self.errors.push(
                        SyntaxError::default()
                            .ctx(ErrorContext::span(
                                SourcePoint::new(self.cur_ln, self.start_col_buf),
                                SourcePoint::new(self.cur_ln, self.cur_col),
                                self.src.clone(),
                            ))
                            .kind(SyntaxErrorKind::Unexpected(SyntaxErrorSource::Character))
                            .msg("identifiers cannot start with a digit"),
                    );
                }
            }
            _ => {}
        }

        self.value_buf.push(ch);
    }

    fn push_keyword_or_ident(&mut self, span: SourceSpan) {
        if let Ok(kind) = Keywords::from_str(&self.value_buf) {
            if matches!(kind, Keywords::True | Keywords::False) {
                self.push_new(
                    span,
                    Tokens::Literal(Literals::Boolean),
                    Some(self.value_buf.clone()),
                )
            } else {
                self.push_new(span, Tokens::Keyword(kind), None);
            }
        } else {
            self.push_new_not_empty(span, self.kind_buf.clone(), self.value_buf.clone());
        }
        self.reset_buffers();
    }

    fn process_complex_punctuation(&mut self, ch: char) {
        match self.punct_buf.as_str() {
            "." => self.recognize_dot(ch),
            "-" => self.recognize_minus(ch),
            _ => {
                self.decompose_complex_punctuation();
            }
        }

        self.punct_buf = String::new();
    }

    fn recognize_dot(&mut self, ch: char) {
        if ch.is_digit(10) {
            self.kind_buf = Tokens::Literal(Literals::Float);
            self.value_buf.push('.');
        } else {
            if self.kind_buf == Tokens::Literal(Literals::Integer) {
                self.push_keyword_or_ident(SourceSpan::one_ln(
                    self.cur_ln,
                    self.start_col_buf,
                    self.cur_col,
                ));
                self.start_col_buf = self.cur_col;
            }

            self.push_new(
                SourceSpan::one_ln(self.cur_ln, self.cur_col - 1, self.cur_col),
                Tokens::Punctuation(Punctuations::MemberSeparator),
                None,
            );

            self.start_col_buf = self.cur_col;
        }
    }

    fn recognize_minus(&mut self, ch: char) {
        if ch.is_digit(10)
            && (self.value_buf.ends_with("e") || self.value_buf.ends_with("E"))
            && let Tokens::Literal(lit) = &self.kind_buf
            && lit.is_number()
        {
            self.kind_buf = Tokens::Literal(Literals::Float);
            self.value_buf.push('-');
        } else {
            self.push_new(
                SourceSpan::one_ln(self.cur_ln, self.cur_col - 1, self.cur_col),
                Tokens::Punctuation(Punctuations::OperatorMinus),
                None,
            );

            self.start_col_buf = self.cur_col;
        }
    }

    fn decompose_complex_punctuation(&mut self) {
        if self.punct_buf.count() == 1 {
            if let Ok(kind) = Punctuations::from_str(&self.punct_buf) {
                self.push_new(
                    SourceSpan::one_ln(self.cur_ln, self.cur_col - 1, self.cur_col),
                    Tokens::Punctuation(kind),
                    None,
                );
            } else {
                self.push_new(
                    SourceSpan::one_ln(self.cur_ln, self.cur_col - 1, self.cur_col),
                    Tokens::Undefined,
                    Some(self.punct_buf.clone()),
                );
            }

            self.start_col_buf = self.cur_col;
            return;
        }

        let mut next_char_idx = 0;
        while next_char_idx < self.punct_buf.count() {
            let mut current_punct_buf = String::new();
            let mut valid_punct_kind = Tokens::Undefined;

            let current_char_idx = next_char_idx;
            for pi in next_char_idx..self.punct_buf.count() {
                let pc = self.punct_buf.clone().chars().nth(pi).unwrap();
                current_punct_buf.push(pc);

                if let Ok(kind) = Punctuations::from_str(&current_punct_buf) {
                    next_char_idx = pi + 1;
                    valid_punct_kind = Tokens::Punctuation(kind);
                }
            }

            let start_col = self.start_col_buf + current_char_idx;
            let end_col = self.start_col_buf + next_char_idx;

            if valid_punct_kind == Tokens::Undefined {
                self.push_new(
                    SourceSpan::one_ln(self.cur_ln, start_col, end_col),
                    valid_punct_kind.clone(),
                    Some(current_punct_buf),
                );

                break;
            }

            self.push_new(
                SourceSpan::one_ln(self.cur_ln, start_col, end_col),
                valid_punct_kind.clone(),
                None,
            );
        }

        self.start_col_buf = self.cur_col;
    }

    fn push(&mut self, token: Token) {
        if token.kind == Tokens::Undefined {
            self.errors.push(
                SyntaxError::default()
                    .ctx(ErrorContext::span(
                        SourcePoint::new(self.cur_ln, self.cur_col - self.punct_buf.count()),
                        SourcePoint::new(self.cur_ln, self.cur_col),
                        self.src.clone(),
                    ))
                    .kind(SyntaxErrorKind::Invalid(SyntaxErrorSource::Token)),
            );
        }

        self.tokens.push(token);
    }

    fn push_new(&mut self, span: SourceSpan, kind: Tokens, value: Option<String>) {
        let token = Token::new(span, kind, value);
        self.push(token);
    }

    fn push_new_not_empty(&mut self, span: SourceSpan, kind: Tokens, value: String) {
        if value.is_empty() {
            return;
        }

        self.push(Token::new(span, kind, Some(value)));
    }

    fn reset_buffers(&mut self) {
        self.value_buf = String::new();
        self.kind_buf = Tokens::Undefined;
    }
}

impl<'a> Default for Lexer<'a> {
    fn default() -> Self {
        Self {
            src: SourceFile::new(PathBuf::new(), &DEFAULT_SRC_CONTENTS),
            tokens: Vec::new(),
            errors: Vec::new(),

            cur_ln: 1,
            cur_col: 0,
            start_col_buf: 1,
            kind_buf: Tokens::Undefined,
            value_buf: String::new(),
            punct_buf: String::new(),
            inside_string: false,
        }
    }
}
