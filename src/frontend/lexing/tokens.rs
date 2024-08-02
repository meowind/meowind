use std::{
    fmt::{self},
    str::FromStr,
};

use crate::{frontend::Loc, utils::colors::*};

#[derive(Clone, Debug)]
pub struct Token {
    pub loc: Loc,
    pub kind: TokenKind,
    pub value: Option<String>,
}

impl Token {
    pub fn new(loc: Loc, kind: TokenKind, value: Option<String>) -> Token {
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

impl ToString for TokenKind {
    fn to_string(&self) -> String {
        match self {
            TokenKind::Literal(kind) => format!("{} literal", kind.to_string()),
            TokenKind::Keyword(kind) => format!("keyword {}", kind.to_string()),
            TokenKind::SimplePunctuation(kind) => format!("\"{}\"", kind.to_char()),
            TokenKind::ComplexPunctuation(kind) => format!("\"{}\"", kind.to_string()),
            _ => format!("{:?}", self),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralKind {
    Integer,
    Float,
    String,
    Boolean,
}

impl LiteralKind {
    pub fn is_number(&self) -> bool {
        matches!(self, LiteralKind::Integer | LiteralKind::Float)
    }
}

impl ToString for LiteralKind {
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
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

    True,
    False,

    Return,
}

impl FromStr for KeywordKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "let" => Ok(Self::Var),
            "func" => Ok(Self::Func),

            "mut" => Ok(Self::Mut),
            "pub" => Ok(Self::Pub),

            "const" => Ok(Self::Const),
            "static" => Ok(Self::Static),

            "true" => Ok(Self::True),
            "false" => Ok(Self::False),

            "return" => Ok(Self::Return),
            _ => Err(()),
        }
    }
}

impl ToString for KeywordKind {
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
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

    pub fn to_char(&self) -> char {
        match self {
            Self::ParenOpen => '(',
            Self::ParenClose => ')',
            Self::BraceOpen => '{',
            Self::BraceClose => '}',
            Self::BracketOpen => '[',
            Self::BracketClose => ']',
            Self::Semicolon => ';',
            Self::Comma => ',',
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

    OperatorAnd,
    OperatorOr,
    OperatorNot,

    Assignment,
    AssignmentPlus,
    AssignmentMinus,
    AssignmentMultiply,
    AssignmentDivide,
    AssignmentModulo,
    AssignmentPower,

    ReturnSeparator,
    MemberSeparator,
    NamespaceSeparator,
    Colon,

    AngleOpen,
    AngleClose,

    Tilde,

    InlineBlock,
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

            "&&" => Ok(Self::OperatorAnd),
            "||" => Ok(Self::OperatorOr),
            "!" => Ok(Self::OperatorNot),

            "=" => Ok(Self::Assignment),
            "+=" => Ok(Self::AssignmentPlus),
            "-=" => Ok(Self::AssignmentMinus),
            "*=" => Ok(Self::AssignmentMultiply),
            "/=" => Ok(Self::AssignmentDivide),
            "%=" => Ok(Self::AssignmentModulo),
            "**=" => Ok(Self::AssignmentPower),

            "->" => Ok(Self::ReturnSeparator),
            "." => Ok(Self::MemberSeparator),
            "::" => Ok(Self::NamespaceSeparator),
            ":" => Ok(Self::Colon),

            "<" => Ok(Self::AngleOpen),
            ">" => Ok(Self::AngleClose),

            "~" => Ok(Self::Tilde),

            "=>" => Ok(Self::InlineBlock),
            _ => Err(()),
        }
    }
}

impl ToString for ComplexPunctuationKind {
    fn to_string(&self) -> String {
        match self {
            Self::OperatorPlus => "+",
            Self::OperatorMinus => "-",
            Self::OperatorMultiply => "*",
            Self::OperatorDivide => "/",
            Self::OperatorModulo => "%",
            Self::OperatorPower => "**",

            Self::OperatorEqual => "==",
            Self::OperatorNotEqual => "!=",
            Self::OperatorLessEqual => "<=",
            Self::OperatorGreaterEqual => ">=",

            Self::OperatorAnd => "&&",
            Self::OperatorOr => "||",
            Self::OperatorNot => "!",

            Self::Assignment => "=",
            Self::AssignmentPlus => "+=",
            Self::AssignmentMinus => "-=",
            Self::AssignmentMultiply => "*=",
            Self::AssignmentDivide => "/=",
            Self::AssignmentModulo => "%=",
            Self::AssignmentPower => "**=",

            Self::ReturnSeparator => "->",
            Self::MemberSeparator => ".",
            Self::NamespaceSeparator => "::",
            Self::Colon => ":",

            Self::AngleOpen => "<",
            Self::AngleClose => ">",

            Self::Tilde => "~",

            Self::InlineBlock => "=>",
        }
        .to_string()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = if let Some(value) = &self.value {
            if self.kind == TokenKind::Literal(LiteralKind::String) {
                format!("\"{BOLD}{}\"{RESET}", value)
            } else {
                format!("{BOLD}{}{RESET}", value)
            }
        } else {
            String::from("")
        };

        let kind = match &self.kind {
            TokenKind::Literal(kind) => format!("{:?}", kind),
            TokenKind::Keyword(kind) => format!("Keyword {BOLD}{}{RESET}", kind.to_string()),
            TokenKind::SimplePunctuation(kind) => {
                format!("{:?}", kind)
            }
            TokenKind::ComplexPunctuation(kind) => {
                format!("{:?}", kind)
            }
            _ => format!("{:?}", self.kind),
        };

        let loc = format!(
            "{GRAY}l:{WHITE}{}{GRAY}, c:{WHITE}{}-{}{RESET}",
            self.loc.ln, self.loc.start_col, self.loc.end_col
        );

        write!(f, "{:>38} | {} {}", loc, kind, value)
    }
}
