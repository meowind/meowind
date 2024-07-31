use super::{block::BlockNode, expressions::ExpressionKind, r#type::TypeNode};

#[derive(Debug)]
pub struct ArgumentNode {
    pub name: String,
    pub r#type: TypeNode,
    pub default: Option<ExpressionKind>,
}

#[derive(Debug)]
pub struct FunctionNode {
    pub name: String,
    pub args: Vec<ArgumentNode>,
    pub r#type: Option<TypeNode>,
    pub body: BlockNode,
}
