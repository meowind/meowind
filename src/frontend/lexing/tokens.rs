use crate::debug;
use std::{fmt, slice, str::FromStr};

pub struct Token {
    ln: usize,
    col: usize,
    kind: TokenKind,
    value: Option<String>,
}

impl Token {
    pub fn new(ln: usize, col: usize, kind: TokenKind, value: Option<String>) -> Token {
        Token {
            ln,
            col,
            kind,
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Literal(LiteralKind),

    Identifier,

    Keyword(KeywordKind),
    SimplePunctuation(SimplePunctuationKind),
    ComplexPunctuation(ComplexPunctuationKind),

    EOF,
    Undefined,
    InvalidIdentifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralKind {
    Integer,
    Float,
}

impl LiteralKind {
    pub fn is_number(&self) -> bool {
        matches!(self, LiteralKind::Integer | LiteralKind::Float)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeywordKind {
    Var,
    Func,
    Mut,
    Pub,
    Const,
    Static,
}

impl FromStr for KeywordKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "var" => Ok(Self::Var),
            "func" => Ok(Self::Func),
            "mut" => Ok(Self::Mut),
            "pub" => Ok(Self::Pub),
            "const" => Ok(Self::Const),
            "static" => Ok(Self::Static),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimplePunctuationKind {
    ParenOpen,
    ParenClose,
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Semicolon,
}

impl SimplePunctuationKind {
    pub fn from_char(ch: char) -> Result<Self, ()> {
        match ch {
            '(' => Ok(Self::ParenOpen),
            ')' => Ok(Self::ParenClose),
            '{' => Ok(Self::BraceOpen),
            '}' => Ok(Self::BraceClose),
            '[' => Ok(Self::BracketOpen),
            ']' => Ok(Self::BracketClose),
            ';' => Ok(Self::Semicolon),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComplexPunctuationKind {
    OperatorPlus,
    OperatorMinus,
    OperatorMultiply,
    OperatorDivide,
    OperatorModulo,
    OperatorPower,

    OperatorEqual,
    OperatorNotEqual,
    OperatorLess,
    OperatorLessEqual,
    OperatorGreater,
    OperatorGreaterEqual,

    Assignment,
    AssignmentPlus,
    AssignmentMinus,
    AssignmentMultiply,
    AssignmentDivide,
    AssignmentModulo,
    AssignmentPower,

    MemberSeparator,
    NamespaceSeparator,
}

impl FromStr for ComplexPunctuationKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Self::OperatorPlus),
            "-" => Ok(Self::OperatorMinus),
            "*" => Ok(Self::OperatorMultiply),
            "/" => Ok(Self::OperatorDivide),
            "%" => Ok(Self::OperatorModulo),
            "**" => Ok(Self::OperatorPower),

            "==" => Ok(Self::OperatorEqual),
            "!=" => Ok(Self::OperatorNotEqual),
            "<" => Ok(Self::OperatorLess),
            ">" => Ok(Self::OperatorGreater),
            "<=" => Ok(Self::OperatorLessEqual),
            ">=" => Ok(Self::OperatorGreaterEqual),

            "=" => Ok(Self::Assignment),
            "+=" => Ok(Self::AssignmentPlus),
            "-=" => Ok(Self::AssignmentMinus),
            "*=" => Ok(Self::AssignmentMultiply),
            "/=" => Ok(Self::AssignmentDivide),
            "%=" => Ok(Self::AssignmentModulo),
            "**=" => Ok(Self::AssignmentPower),

            "." => Ok(Self::MemberSeparator),
            "::" => Ok(Self::NamespaceSeparator),
            _ => Err(()),
        }
    }
}

pub struct Tokens {
    pub vector: Vec<Token>,
}

impl Tokens {
    pub fn new() -> Tokens {
        Tokens { vector: Vec::new() }
    }

    pub fn push(&mut self, token: Token) {
        debug!("== LEXER PUSHING TOKEN ==\n{}\n", token);
        self.vector.push(token);
    }

    pub fn push_not_empty(&mut self, token: Token) {
        if let Some(value) = &token.value {
            if value.is_empty() {
                return;
            }
        }

        self.push(token);
    }

    pub fn push_new(&mut self, ln: usize, col: usize, kind: TokenKind, value: Option<String>) {
        let token = Token::new(ln, col, kind, value);
        debug!("== LEXER PUSHING TOKEN ==\n{}\n", token);
        self.vector.push(token);
    }

    pub fn push_new_not_empty(&mut self, ln: usize, col: usize, kind: TokenKind, value: String) {
        if value.is_empty() {
            return;
        }

        self.push(Token::new(ln, col, kind, Some(value)));
    }

    pub fn iter(&self) -> slice::Iter<Token> {
        self.vector.iter()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = if let Some(value) = &self.value {
            format!(" \"{}\"", value)
        } else {
            String::from("")
        };

        write!(
            f,
            "(ln: {}, col: {}) {:?}{}",
            self.ln, self.col, self.kind, value
        )
    }
}
