use std::isize;

use crate::frontend::lexing::{ComplexPunctuationKind, LiteralKind};

#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionNode {
    pub kind: ExpressionKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Literal {
        kind: LiteralKind,
        value: String,
    },

    Identifier {
        name: String,
    },

    Call {
        res: Box<ExpressionNode>,
        args: Vec<ExpressionNode>,
    },

    Resolution {
        left: Box<ExpressionNode>,
        right: Box<ExpressionNode>,
        kind: ResolutionExpressionKind,
    },

    Binary {
        kind: BinaryExpressionKind,
        left: Box<ExpressionNode>,
        op: ComplexPunctuationKind,
        right: Box<ExpressionNode>,
    },

    Unary {
        kind: UnaryExpressionKind,
        op: ComplexPunctuationKind,
        right: Box<ExpressionNode>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryExpressionKind {
    ArithmeticNegation,
    LogicalNegation,
}

impl UnaryExpressionKind {
    pub fn from_punct(opr: &ComplexPunctuationKind) -> Result<UnaryExpressionKind, ()> {
        match opr {
            ComplexPunctuationKind::OperatorMinus => Ok(UnaryExpressionKind::ArithmeticNegation),
            ComplexPunctuationKind::OperatorNot => Ok(UnaryExpressionKind::LogicalNegation),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryExpressionKind {
    LogicalAnd,
    LogicalOr,
    Equality,
    Relational,
    Additive,
    Multiplicative,
    Exponential,
}

impl BinaryExpressionKind {
    pub fn from_punct(opr: &ComplexPunctuationKind) -> Result<BinaryExpressionKind, ()> {
        match opr {
            ComplexPunctuationKind::OperatorAnd => Ok(BinaryExpressionKind::LogicalAnd),

            ComplexPunctuationKind::OperatorOr => Ok(BinaryExpressionKind::LogicalOr),

            ComplexPunctuationKind::OperatorEqual => Ok(BinaryExpressionKind::Equality),
            ComplexPunctuationKind::OperatorNotEqual => Ok(BinaryExpressionKind::Equality),

            ComplexPunctuationKind::AngleOpen => Ok(BinaryExpressionKind::Relational),
            ComplexPunctuationKind::AngleClose => Ok(BinaryExpressionKind::Relational),
            ComplexPunctuationKind::OperatorLessEqual => Ok(BinaryExpressionKind::Relational),
            ComplexPunctuationKind::OperatorGreaterEqual => Ok(BinaryExpressionKind::Relational),

            ComplexPunctuationKind::OperatorPlus => Ok(BinaryExpressionKind::Additive),
            ComplexPunctuationKind::OperatorMinus => Ok(BinaryExpressionKind::Additive),

            ComplexPunctuationKind::OperatorMultiply => Ok(BinaryExpressionKind::Multiplicative),
            ComplexPunctuationKind::OperatorDivide => Ok(BinaryExpressionKind::Multiplicative),
            ComplexPunctuationKind::OperatorModulo => Ok(BinaryExpressionKind::Multiplicative),

            ComplexPunctuationKind::OperatorPower => Ok(BinaryExpressionKind::Exponential),

            _ => Err(()),
        }
    }

    pub fn lowest() -> BinaryExpressionKind {
        BinaryExpressionKind::LogicalAnd
    }

    pub fn from_precedence(precedence: isize) -> Result<BinaryExpressionKind, ()> {
        match precedence {
            -4 => Ok(BinaryExpressionKind::LogicalAnd),
            -3 => Ok(BinaryExpressionKind::LogicalOr),
            -2 => Ok(BinaryExpressionKind::Equality),
            -1 => Ok(BinaryExpressionKind::Relational),
            0 => Ok(BinaryExpressionKind::Additive),
            1 => Ok(BinaryExpressionKind::Multiplicative),
            2 => Ok(BinaryExpressionKind::Exponential),
            _ => Err(()),
        }
    }

    pub fn precedence(&self) -> isize {
        match self {
            BinaryExpressionKind::LogicalAnd => -4,
            BinaryExpressionKind::LogicalOr => -3,
            BinaryExpressionKind::Equality => -2,
            BinaryExpressionKind::Relational => -1,
            BinaryExpressionKind::Additive => 0,
            BinaryExpressionKind::Multiplicative => 1,
            BinaryExpressionKind::Exponential => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionExpressionKind {
    Namespace,
    Member,
}
