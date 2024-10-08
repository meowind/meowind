use super::{bodies::BodyNode, expressions::ExpressionNode, types::TypeNode};

#[derive(Debug)]
pub struct ArgumentNode {
    pub name: String,
    pub r#type: Option<TypeNode>,
    pub default: Option<ExpressionNode>,
}

#[derive(Debug)]
pub struct FunctionNode {
    pub name: String,
    pub args: Vec<ArgumentNode>,
    pub r#type: Option<TypeNode>,
    pub return_var: Option<String>,
    pub body: BodyNode,
}
