use std::isize;

use crate::frontend::lexing::{Assignments, Literals, Punctuations};

#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionNode {
    pub kind: ExpressionKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Literal {
        kind: Literals,
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
        op: Punctuations,
        right: Box<ExpressionNode>,
    },

    Unary {
        kind: UnaryExpressionKind,
        op: Punctuations,
        right: Box<ExpressionNode>,
    },

    Assignment {
        left: Box<ExpressionNode>,
        op: Assignments,
        right: Box<ExpressionNode>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryExpressionKind {
    ArithmeticNegation,
    LogicalNegation,
}

impl UnaryExpressionKind {
    pub fn from_punct(opr: &Punctuations) -> Result<UnaryExpressionKind, ()> {
        match opr {
            Punctuations::OperatorMinus => Ok(UnaryExpressionKind::ArithmeticNegation),
            Punctuations::OperatorNot => Ok(UnaryExpressionKind::LogicalNegation),
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
    pub fn from_punct(opr: &Punctuations) -> Result<BinaryExpressionKind, ()> {
        match opr {
            Punctuations::OperatorAnd => Ok(BinaryExpressionKind::LogicalAnd),

            Punctuations::OperatorOr => Ok(BinaryExpressionKind::LogicalOr),

            Punctuations::OperatorEqual => Ok(BinaryExpressionKind::Equality),
            Punctuations::OperatorNotEqual => Ok(BinaryExpressionKind::Equality),

            Punctuations::AngleOpen => Ok(BinaryExpressionKind::Relational),
            Punctuations::AngleClose => Ok(BinaryExpressionKind::Relational),
            Punctuations::OperatorLessEqual => Ok(BinaryExpressionKind::Relational),
            Punctuations::OperatorGreaterEqual => Ok(BinaryExpressionKind::Relational),

            Punctuations::OperatorPlus => Ok(BinaryExpressionKind::Additive),
            Punctuations::OperatorMinus => Ok(BinaryExpressionKind::Additive),

            Punctuations::OperatorMultiply => Ok(BinaryExpressionKind::Multiplicative),
            Punctuations::OperatorDivide => Ok(BinaryExpressionKind::Multiplicative),
            Punctuations::OperatorModulo => Ok(BinaryExpressionKind::Multiplicative),

            Punctuations::OperatorPower => Ok(BinaryExpressionKind::Exponential),

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
