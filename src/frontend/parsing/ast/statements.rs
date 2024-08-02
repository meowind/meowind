use super::{expressions::ExpressionNode, functions::FunctionNode, r#type::TypeNode};

#[derive(Debug)]
pub struct StatementNode {
    pub kind: StatementKind,
}

#[derive(Debug)]
pub enum StatementKind {
    Expression(ExpressionNode),
    VariableDeclaration(VariableDeclarationNode),
    FunctionDeclaration(FunctionNode),
    Return(ExpressionNode),
}

#[derive(Debug)]
pub struct VariableDeclarationNode {
    pub name: String,
    pub r#type: Option<TypeNode>,
    pub value: Option<ExpressionNode>,
    pub mutable: bool,
}
