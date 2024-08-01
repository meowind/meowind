use std::isize;

use crate::frontend::lexing::{ComplexPunctuationKind, LiteralKind};

#[derive(Debug, Clone)]
pub struct ExpressionNode {
    pub kind: ExpressionKind,
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    Literal {
        kind: LiteralKind,
        value: String,
    },

    Binary {
        kind: BinaryExpressionKind,
        left: Box<ExpressionNode>,
        op: ComplexPunctuationKind,
        right: Box<ExpressionNode>,
    },

    Unary {
        op: ComplexPunctuationKind,
        right: Box<ExpressionNode>,
    },

    Call {
        func: String,
        args: Vec<ExpressionNode>,
    },

    Array {
        elements: Vec<ExpressionNode>,
    },

    Identifier {
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryExpressionKind {
    Additive,
    Multiplicative,
    Exponential,
}

impl BinaryExpressionKind {
    pub fn from_punct(opr: &ComplexPunctuationKind) -> Result<BinaryExpressionKind, ()> {
        match opr {
            ComplexPunctuationKind::OperatorPlus => Ok(BinaryExpressionKind::Additive),
            ComplexPunctuationKind::OperatorMinus => Ok(BinaryExpressionKind::Additive),

            ComplexPunctuationKind::OperatorMultiply => Ok(BinaryExpressionKind::Multiplicative),
            ComplexPunctuationKind::OperatorDivide => Ok(BinaryExpressionKind::Multiplicative),
            ComplexPunctuationKind::OperatorModulo => Ok(BinaryExpressionKind::Multiplicative),

            ComplexPunctuationKind::OperatorPower => Ok(BinaryExpressionKind::Exponential),

            _ => Err(()),
        }
    }

    pub fn from_precedence(precedence: isize) -> Result<BinaryExpressionKind, ()> {
        match precedence {
            0 => Ok(BinaryExpressionKind::Additive),
            1 => Ok(BinaryExpressionKind::Multiplicative),
            2 => Ok(BinaryExpressionKind::Exponential),
            _ => Err(()),
        }
    }

    pub fn precedence(&self) -> isize {
        match self {
            BinaryExpressionKind::Additive => 0,
            BinaryExpressionKind::Multiplicative => 1,
            BinaryExpressionKind::Exponential => 2,
        }
    }
}
