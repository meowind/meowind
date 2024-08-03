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
    If(IfNode),
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
pub struct IfNode {
    pub kind: IfKind,
    pub body: BlockNode,
}

#[derive(Debug)]
pub enum IfKind {
    If {
        cond: ExpressionNode,
        r#else: Option<Box<IfNode>>,
    },
    Else,
}

#[derive(Debug)]
pub struct WhileLoopNode {
    pub kind: WhileLoopKind,
    pub body: BlockNode,
}

#[derive(Debug)]
pub enum WhileLoopKind {
    While {
        cond: ExpressionNode,
        r#else: Option<Box<WhileLoopNode>>,
    },
    Else,
}
