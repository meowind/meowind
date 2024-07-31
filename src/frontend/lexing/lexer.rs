// TODO: refactor errors

use super::tokens::{
    ComplexPunctuationKind::{self, *},
    KeywordKind,
    LiteralKind::*,
    SimplePunctuationKind, Token,
    TokenKind::{self, *},
};
use crate::{
    errors::{
        context::ErrorContextBuilder,
        syntax::{SyntaxError, SyntaxErrorKind, SyntaxErrorSource},
        MeowindErrorList,
    },
    frontend::Loc,
    structs::{ScriptSource, DEFAULT_SRC_CONTENTS},
};
use std::{fmt, path::PathBuf, str::FromStr, string::String as StdString};
use unicode_segmentation::UnicodeSegmentation;

pub struct Lexer<'a> {
    pub src: ScriptSource<'a>,

    pub tokens: Vec<Token>,
    pub errors: MeowindErrorList<SyntaxError>,

    cur_ln: usize,
    cur_col: usize,
    start_col_buf: usize,
    kind_buf: TokenKind,
    value_buf: LexerValueBuffer,
    punct_buf: LexerValueBuffer,

    inside_string: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: ScriptSource) -> Lexer {
        Lexer {
            src: source,

            ..Default::default()
        }
    }

    pub fn tokenize(source: ScriptSource<'a>) -> Lexer<'a> {
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
                    .ctx(
                        ErrorContextBuilder::col(self.cur_col)
                            .from_src_and_ln(&self.src, self.cur_ln)
                            .build(),
                    )
                    .kind(SyntaxErrorKind::Expected(SyntaxErrorSource::Character))
                    .msg("expected double quote to close string literal"),
            );
        }

        if !self.punct_buf.is_empty() {
            self.process_complex_punctuation('\n');
        } else {
            self.push_keyword_or_ident(Loc::new(self.cur_ln, self.start_col_buf, self.cur_col));
        }

        self.push_new(Loc::new(self.cur_ln, self.cur_col, self.cur_col), EOF, None);
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
                        .ctx(
                            ErrorContextBuilder::col(self.cur_col)
                                .from_src_and_ln(&self.src, self.cur_ln)
                                .build(),
                        )
                        .kind(SyntaxErrorKind::Expected(SyntaxErrorSource::Character))
                        .msg("regular string literals cannot be over multiple lines"),
                );

                self.inside_string = false;
                self.reset_buffers();
            } else {
                if !self.punct_buf.is_empty() {
                    self.process_complex_punctuation('\n');
                } else {
                    self.push_keyword_or_ident(Loc::new(
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
                    self.push_keyword_or_ident(Loc::new(
                        self.cur_ln,
                        self.start_col_buf,
                        self.cur_col,
                    ));
                }

                self.start_col_buf = self.cur_col;
                self.kind_buf = Literal(String);
            } else {
                self.push_new(
                    Loc::new(self.cur_ln, self.start_col_buf, self.cur_col + 1),
                    Literal(String),
                    Some(self.value_buf.value.clone()),
                );
                self.reset_buffers();

                self.start_col_buf = self.cur_col + 1;
                self.kind_buf = Undefined;
            }

            return;
        }

        if self.inside_string {
            self.value_buf.push(ch);
            return;
        }

        if let Ok(kind) = SimplePunctuationKind::from_char(ch) {
            if !self.punct_buf.is_empty() {
                self.process_complex_punctuation(ch);
            } else {
                self.push_keyword_or_ident(Loc::new(self.cur_ln, self.start_col_buf, self.cur_col));
            }

            self.push_new(
                Loc::new(self.cur_ln, self.cur_col, self.cur_col + 1),
                SimplePunctuation(kind),
                None,
            );

            self.start_col_buf = self.cur_col + 1;
            return;
        }

        if ch.is_ascii_punctuation() && ch != '_' {
            if self.punct_buf.is_empty() && self.kind_buf != Literal(Integer) {
                self.push_keyword_or_ident(Loc::new(self.cur_ln, self.start_col_buf, self.cur_col));
                self.start_col_buf = self.cur_col;
            }

            self.punct_buf.push(ch);
            return;
        }

        if !self.punct_buf.is_empty() {
            self.process_complex_punctuation(ch);
        }

        if ch.is_whitespace() {
            self.push_keyword_or_ident(Loc::new(self.cur_ln, self.start_col_buf, self.cur_col));

            self.start_col_buf = self.cur_col + 1;
            return;
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

                    self.errors.push(
                        SyntaxError::default()
                            .ctx(
                                ErrorContextBuilder::span(self.start_col_buf, self.cur_col)
                                    .from_src_and_ln(&self.src, self.cur_ln)
                                    .build(),
                            )
                            .kind(SyntaxErrorKind::Unexpected(SyntaxErrorSource::Character))
                            .msg("identifiers cannot start with a digit"),
                    );
                }
            }
            _ => {}
        }

        self.value_buf.push(ch);
    }

    fn push_keyword_or_ident(&mut self, loc: Loc) {
        if let Ok(kind) = KeywordKind::from_str(&self.value_buf.value) {
            self.push_new(
                Loc::new(loc.ln, loc.start_col, loc.end_col),
                Keyword(kind),
                None,
            );
        } else {
            self.push_new_not_empty(
                Loc::new(loc.ln, loc.start_col, loc.end_col),
                self.kind_buf.clone(),
                self.value_buf.value.clone(),
            );
        }
        self.reset_buffers();
    }

    fn process_complex_punctuation(&mut self, ch: char) {
        match self.punct_buf.value.as_str() {
            "." => self.recognize_dot(ch),
            "-" => self.recognize_minus(ch),
            _ => {
                self.decompose_complex_punctuation();
            }
        }

        self.punct_buf = LexerValueBuffer::new();
    }

    fn recognize_dot(&mut self, ch: char) {
        if ch.is_digit(10) {
            self.kind_buf = Literal(Float);
            self.value_buf.push('.');
        } else {
            if self.kind_buf == Literal(Integer) {
                self.push_keyword_or_ident(Loc::new(self.cur_ln, self.start_col_buf, self.cur_col));
                self.start_col_buf = self.cur_col;
            }

            self.push_new(
                Loc::new(self.cur_ln, self.cur_col - 1, self.cur_col),
                ComplexPunctuation(MemberSeparator),
                None,
            );

            self.start_col_buf = self.cur_col;
        }
    }

    fn recognize_minus(&mut self, ch: char) {
        if ch.is_digit(10)
            && (self.value_buf.value.ends_with("e") || self.value_buf.value.ends_with("E"))
            && let Literal(lit) = &self.kind_buf
            && lit.is_number()
        {
            self.kind_buf = Literal(Float);
            self.value_buf.push('-');
        } else {
            self.push_new(
                Loc::new(self.cur_ln, self.cur_col - 1, self.cur_col),
                ComplexPunctuation(OperatorMinus),
                None,
            );

            self.start_col_buf = self.cur_col;
        }
    }

    fn decompose_complex_punctuation(&mut self) {
        if self.punct_buf.count() == 1 {
            if let Ok(kind) = ComplexPunctuationKind::from_str(&self.punct_buf.value) {
                self.push_new(
                    Loc::new(self.cur_ln, self.cur_col - 1, self.cur_col),
                    ComplexPunctuation(kind),
                    None,
                );
            } else {
                self.push_new(
                    Loc::new(self.cur_ln, self.cur_col - 1, self.cur_col),
                    Undefined,
                    Some(self.punct_buf.value.clone()),
                );
            }

            self.start_col_buf = self.cur_col;
            return;
        }

        let mut next_char_idx = 0;
        while next_char_idx < self.punct_buf.count() {
            let mut current_punct_buf = StdString::new();
            let mut valid_punct_kind = Undefined;

            let current_char_idx = next_char_idx;
            for pi in next_char_idx..self.punct_buf.count() {
                let pc = self.punct_buf.value.clone().chars().nth(pi).unwrap();
                current_punct_buf.push(pc);

                if let Ok(kind) = ComplexPunctuationKind::from_str(&current_punct_buf) {
                    next_char_idx = pi + 1;
                    valid_punct_kind = ComplexPunctuation(kind);
                }
            }

            let start_col = self.start_col_buf + current_char_idx;
            let end_col = self.start_col_buf + next_char_idx;

            if valid_punct_kind == Undefined {
                self.push_new(
                    Loc::new(self.cur_ln, start_col, end_col),
                    valid_punct_kind.clone(),
                    Some(current_punct_buf),
                );

                break;
            }

            self.push_new(
                Loc::new(self.cur_ln, start_col, end_col),
                valid_punct_kind.clone(),
                None,
            );
        }

        self.start_col_buf = self.cur_col;
    }

    fn push(&mut self, token: Token) {
        if token.kind == Undefined {
            self.errors.push(
                SyntaxError::default()
                    .ctx(
                        ErrorContextBuilder::span(
                            self.cur_col - self.punct_buf.count(),
                            self.cur_col,
                        )
                        .from_src_and_ln(&self.src, self.cur_ln)
                        .build(),
                    )
                    .kind(SyntaxErrorKind::Invalid(SyntaxErrorSource::Token)),
            );
        }

        self.tokens.push(token);
    }

    fn push_new(&mut self, loc: Loc, kind: TokenKind, value: Option<StdString>) {
        let token = Token::new(loc, kind, value);
        self.push(token);
    }

    fn push_new_not_empty(&mut self, loc: Loc, kind: TokenKind, value: StdString) {
        if value.is_empty() {
            return;
        }

        self.push(Token::new(loc, kind, Some(value)));
    }

    fn reset_buffers(&mut self) {
        self.value_buf = LexerValueBuffer::new();
        self.kind_buf = Undefined;
    }
}

impl<'a> Default for Lexer<'a> {
    fn default() -> Self {
        Self {
            src: ScriptSource::new(PathBuf::new(), DEFAULT_SRC_CONTENTS),
            tokens: Vec::new(),
            errors: MeowindErrorList::new(),

            cur_ln: 1,
            cur_col: 0,
            start_col_buf: 1,
            kind_buf: Undefined,
            value_buf: LexerValueBuffer::new(),
            punct_buf: LexerValueBuffer::new(),
            inside_string: false,
        }
    }
}

pub struct LexerValueBuffer {
    value: StdString,
}

impl LexerValueBuffer {
    pub fn new() -> LexerValueBuffer {
        LexerValueBuffer {
            value: StdString::new(),
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
