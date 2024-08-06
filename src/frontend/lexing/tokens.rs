use std::{
    fmt::{self},
    str::FromStr,
};

use crate::{source::SourceSpan, utils::colors::*};

#[derive(Clone, Debug)]
pub struct Token {
    pub span: SourceSpan,
    pub kind: Tokens,
    pub value: Option<String>,
}

impl Token {
    pub fn new(span: SourceSpan, kind: Tokens, value: Option<String>) -> Token {
        Token { span, kind, value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tokens {
    Literal(Literals),

    Identifier,

    Keyword(Keywords),
    Punctuation(Punctuations),

    EOF,
    Undefined,
    InvalidIdentifier,
}

impl ToString for Tokens {
    fn to_string(&self) -> String {
        match self {
            Tokens::Literal(kind) => format!("{} literal", kind.to_string()),
            Tokens::Keyword(kind) => format!("keyword {}", kind.to_string()),
            Tokens::Punctuation(kind) => format!("\"{}\"", kind.to_string()),
            _ => format!("{:?}", self),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literals {
    Integer,
    Float,
    String,
    Boolean,
}

impl Literals {
    pub fn is_number(&self) -> bool {
        matches!(self, Literals::Integer | Literals::Float)
    }
}

impl ToString for Literals {
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keywords {
    Var,
    Func,

    Mut,
    Pub,

    Const,
    Static,

    True,
    False,

    Return,

    While,
    If,
    Else,
}

impl FromStr for Keywords {
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

            "while" => Ok(Self::While),
            "if" => Ok(Self::If),
            "else" => Ok(Self::Else),
            _ => Err(()),
        }
    }
}

impl ToString for Keywords {
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Punctuations {
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

    Assignment(Assignments),

    ReturnSeparator,
    MemberSeparator,
    NamespaceSeparator,
    Colon,

    AngleOpen,
    AngleClose,

    Tilde,

    InlineBody,

    ParenOpen,
    ParenClose,

    BraceOpen,
    BraceClose,

    BracketOpen,
    BracketClose,

    Semicolon,
    Comma,
}

impl FromStr for Punctuations {
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

            "->" => Ok(Self::ReturnSeparator),
            "." => Ok(Self::MemberSeparator),
            "::" => Ok(Self::NamespaceSeparator),
            ":" => Ok(Self::Colon),

            "<" => Ok(Self::AngleOpen),
            ">" => Ok(Self::AngleClose),

            "~" => Ok(Self::Tilde),

            "=>" => Ok(Self::InlineBody),

            "(" => Ok(Self::ParenOpen),
            ")" => Ok(Self::ParenClose),

            "{" => Ok(Self::BraceOpen),
            "}" => Ok(Self::BraceClose),

            "[" => Ok(Self::BracketOpen),
            "]" => Ok(Self::BracketClose),

            ";" => Ok(Self::Semicolon),
            "," => Ok(Self::Comma),

            _ => match Assignments::from_str(s) {
                Ok(kind) => Ok(Self::Assignment(kind)),
                Err(_) => Err(()),
            },
        }
    }
}

impl ToString for Punctuations {
    fn to_string(&self) -> String {
        if let Self::Assignment(kind) = self {
            return kind.to_string();
        }

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

            Self::ReturnSeparator => "->",
            Self::MemberSeparator => ".",
            Self::NamespaceSeparator => "::",
            Self::Colon => ":",

            Self::AngleOpen => "<",
            Self::AngleClose => ">",

            Self::Tilde => "~",

            Self::InlineBody => "=>",

            Self::ParenOpen => "(",
            Self::ParenClose => ")",

            Self::BraceOpen => "{",
            Self::BraceClose => "}",

            Self::BracketOpen => "[",
            Self::BracketClose => "]",

            Self::Semicolon => ";",
            Self::Comma => ",",

            Self::Assignment(_) => unreachable!(),
        }
        .to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Assignments {
    Straight,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
}

impl FromStr for Assignments {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "=" => Ok(Self::Straight),
            "+=" => Ok(Self::Add),
            "-=" => Ok(Self::Subtract),
            "*=" => Ok(Self::Multiply),
            "/=" => Ok(Self::Divide),
            "%=" => Ok(Self::Modulo),
            "**=" => Ok(Self::Power),
            _ => Err(()),
        }
    }
}

impl ToString for Assignments {
    fn to_string(&self) -> String {
        match self {
            Self::Straight => "=",
            Self::Add => "+=",
            Self::Subtract => "-=",
            Self::Multiply => "*=",
            Self::Divide => "/=",
            Self::Modulo => "%=",
            Self::Power => "**=",
        }
        .to_string()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = if let Some(value) = &self.value {
            if self.kind == Tokens::Literal(Literals::String) {
                format!("\"{BOLD}{}\"{RESET}", value)
            } else {
                format!("{BOLD}{}{RESET}", value)
            }
        } else {
            String::from("")
        };

        let kind = match &self.kind {
            Tokens::Literal(kind) => format!("{:?}", kind),
            Tokens::Keyword(kind) => format!("Keyword {BOLD}{}{RESET}", kind.to_string()),
            Tokens::Punctuation(kind) => {
                format!("{:?}", kind)
            }
            _ => format!("{:?}", self.kind),
        };

        let loc = format!(
            "{GRAY}l:{WHITE}{}{GRAY}, c:{WHITE}{}-{}{RESET}",
            self.span.start.ln, self.span.start.col, self.span.end.col
        );

        write!(f, "{:>38} | {} {}", loc, kind, value)
    }
}
