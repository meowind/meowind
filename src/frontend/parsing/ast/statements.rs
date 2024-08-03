use super::{
    block::BlockNode, expressions::ExpressionNode, functions::FunctionNode, r#type::TypeNode,
};

#[derive(Debug)]
pub struct StatementNode {
    pub kind: StatementKind,
}

#[derive(Debug)]
pub enum StatementKind {
    Expression(ExpressionNode),
    VariableDeclaration(VariableDeclarationNode),
    FunctionDeclaration(FunctionNode),
    WhileLoop(WhileLoopNode),
    Return(ExpressionNode),
}

#[derive(Debug)]
pub struct VariableDeclarationNode {
    pub name: String,
    pub r#type: Option<TypeNode>,
    pub value: Option<ExpressionNode>,
    pub mutable: bool,
}

#[derive(Debug)]
pub struct WhileLoopNode {
    pub cond: ExpressionNode,
    pub body: BlockNode,
    pub never: Option<BlockNode>,
}
