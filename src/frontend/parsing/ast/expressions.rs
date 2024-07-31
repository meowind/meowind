use crate::frontend::lexing::{ComplexPunctuationKind, LiteralKind};

#[derive(Debug)]
pub struct ExpressionNode {
    pub kind: ExpressionKind,
}

#[derive(Debug)]
pub enum ExpressionKind {
    Literal {
        kind: LiteralKind,
        value: String,
    },

    Binary {
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
}
