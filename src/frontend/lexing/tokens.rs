use std::{fmt, str::FromStr};

use crate::frontend::Location;

pub struct Token {
    pub loc: Location,
    pub kind: TokenKind,
    pub value: Option<String>,
}

impl Token {
    pub fn new(loc: Location, kind: TokenKind, value: Option<String>) -> Token {
        Token { loc, kind, value }
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
    String,
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
    Use,
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
            "use" => Ok(Self::Use),
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
    Comma,
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
            ',' => Ok(Self::Comma),
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
    OperatorLessEqual,
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

    AngleOpen,
    AngleClose,
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

            "<" => Ok(Self::AngleOpen),
            ">" => Ok(Self::AngleClose),
            _ => Err(()),
        }
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
            self.loc.ln, self.loc.start_col, self.kind, value
        )
    }
}
