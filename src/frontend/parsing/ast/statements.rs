use super::{expressions::ExpressionNode, functions::FunctionNode, r#type::TypeNode};

#[derive(Debug)]
pub struct StatementNode {
    kind: StatementKind,
}

#[derive(Debug)]
pub enum StatementKind {
    VariableDeclaration(VariableDeclarationNode),
    FunctionDeclaration(FunctionNode),
}

#[derive(Debug)]
pub struct VariableDeclarationNode {
    name: String,
    r#type: Option<TypeNode>,
    value: VariableValueKind,
    mutable: bool,
}

#[derive(Debug)]
pub enum VariableValueKind {
    Some(ExpressionNode),
    Default,
    None,
}
