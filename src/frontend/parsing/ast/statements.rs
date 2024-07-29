use super::{arg::ArgumentNode, block::BlockNode, expressions::ExpressionNode, r#type::TypeNode};

#[derive(Debug)]
pub struct StatementNode {
    kind: StatementKind,
}

#[derive(Debug)]
pub enum StatementKind {
    VariableDeclaration {
        name: String,
        r#type: Option<TypeNode>,
        value: VariableValueKind,
        mutable: bool,
    },
    FunctionDeclaration {
        name: String,
        args: Vec<ArgumentNode>,
        r#type: Option<TypeNode>,
        body: BlockNode,
    },
}

#[derive(Debug)]
pub enum VariableValueKind {
    Some(ExpressionNode),
    Default,
    None,
}
