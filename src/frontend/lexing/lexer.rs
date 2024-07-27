// TODO: undefined errors

use super::tokens::{
    ComplexPunctuationKind::{self, *},
    KeywordKind,
    LiteralKind::*,
    SimplePunctuationKind, Token,
    TokenKind::{self, *},
};
use crate::{
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

    tokens: Vec<Token>,
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
            tokens: Vec::new(),
            errors: MeowindErrorList::new(),

            ln: 1,
            col: 0,
            kind_buf: Undefined,
            value_buf: LexerValueBuffer::new(),
            punct_buf: LexerValueBuffer::new(),
        }
    }

    pub fn tokenize(&mut self) -> (&Vec<Token>, &MeowindErrorList<SyntaxError>) {
        self.tokens.clear();
        self.errors.vector.clear();
        self.reset_buffers();

        for ch in self.src.contents.chars() {
            self.iteration(ch);
        }
        self.iteration('\0');

        self.push_new(self.ln, self.col, EOF, None);
        return (&self.tokens, &self.errors);
    }

    fn iteration(&mut self, ch: char) {
        if ch == '\n' {
            self.ln += 1;
            self.col = 0;

            self.process_keyword();
            return;
        }
        self.col += 1;

        if let Ok(kind) = SimplePunctuationKind::from_char(ch) {
            if !self.punct_buf.is_empty() {
                self.process_complex_punctuation(ch);
            } else {
                self.push_not_empty(Token::new(
                    self.ln,
                    self.col - self.value_buf.count(),
                    self.kind_buf.clone(),
                    Some(self.value_buf.value.clone()),
                ));
            }
            self.reset_buffers();

            self.push_new(self.ln, self.col, SimplePunctuation(kind), None);
            return;
        }

        if ch.is_ascii_punctuation() && ch != '_' {
            self.punct_buf.push(ch);
            return;
        }

        if !self.punct_buf.is_empty() {
            self.process_complex_punctuation(ch);
        }

        if ch.is_whitespace() {
            self.process_keyword();
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
    }

    fn process_keyword(&mut self) {
        if let Ok(kind) = KeywordKind::from_str(&self.value_buf.value) {
            self.push_new(
                self.ln,
                self.col - self.value_buf.count(),
                Keyword(kind),
                None,
            );
        } else {
            self.push_new_not_empty(
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
                self.push_new_not_empty(
                    self.ln,
                    self.col - self.value_buf.count() - self.punct_buf.count(),
                    self.kind_buf.clone(),
                    self.value_buf.value.clone(),
                );
                self.reset_buffers();

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
            self.push_new_not_empty(
                self.ln,
                self.col - self.value_buf.count(),
                self.kind_buf.clone(),
                self.value_buf.value.clone(),
            );
            self.reset_buffers();

            self.push_new(
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
            self.kind_buf = Literal(Float);
            self.value_buf.push('-');
        } else {
            self.push_new_not_empty(
                self.ln,
                self.col - self.value_buf.count() - 1,
                self.kind_buf.clone(),
                self.value_buf.value.clone(),
            );
            self.reset_buffers();

            self.push_new(
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
                self.push_new(self.ln, self.col - 1, ComplexPunctuation(kind), None);

                return;
            } else {
                self.push_new(
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
                self.push_new(
                    self.ln,
                    self.col - self.punct_buf.count() + from_char_idx,
                    valid_punct_kind.clone(),
                    Some(current_punct_buf),
                );

                break;
            }

            self.push_new(
                self.ln,
                self.col - self.punct_buf.count() + from_char_idx,
                valid_punct_kind.clone(),
                None,
            );
        }
    }

    fn push(&mut self, token: Token) {
        if token.kind == Undefined {
            self.errors.push(SyntaxError::new_with_context(
                SyntaxErrorKind::InvalidToken,
                "got undefined token",
                self.ln,
                self.current_line(),
                self.col - self.value_buf.count() - self.punct_buf.count(),
                self.col,
                self.src.path.clone(),
            ))
        }

        self.tokens.push(token);
    }

    fn push_not_empty(&mut self, token: Token) {
        if let Some(value) = &token.value {
            if value.is_empty() {
                return;
            }
        }

        self.push(token);
    }

    fn push_new(&mut self, ln: usize, col: usize, kind: TokenKind, value: Option<String>) {
        let token = Token::new(ln, col, kind, value);
        self.push(token);
    }

    fn push_new_not_empty(&mut self, ln: usize, col: usize, kind: TokenKind, value: String) {
        if value.is_empty() {
            return;
        }

        self.push(Token::new(ln, col, kind, Some(value)));
    }

    fn reset_buffers(&mut self) {
        self.value_buf = LexerValueBuffer::new();
        self.kind_buf = Undefined;
    }

    fn current_line(&self) -> String {
        self.src.lines[self.ln - 1].to_string()
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
