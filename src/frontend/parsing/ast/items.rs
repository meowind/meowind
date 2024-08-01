use super::{expressions::ExpressionNode, functions::FunctionNode, r#type::TypeNode};

#[derive(Debug)]
pub struct ItemNode {
    pub kind: ItemKind,
    pub public: bool,
}

#[derive(Debug)]
pub enum ItemKind {
    Constant(ConstantNode),
    Static(StaticNode),
    Function(FunctionNode),
}

#[derive(Debug)]
pub struct ConstantNode {
    pub name: String,
    pub r#type: TypeNode,
    pub value: ExpressionNode,
}

#[derive(Debug)]
pub struct StaticNode {
    pub name: String,
    pub r#type: Option<TypeNode>,
    pub value: ExpressionNode,
    pub mutable: bool,
}
