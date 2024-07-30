use super::{block::BlockNode, expressions::ExpressionNode, r#type::TypeNode};

#[derive(Debug)]
pub struct ItemNode {
    pub name: String,
    pub kind: ItemKind,
    pub public: bool,
}

#[derive(Debug)]
pub enum ItemKind {
    Constant {
        r#type: TypeNode,
        value: ExpressionNode,
    },

    Static {
        r#type: TypeNode,
        value: ExpressionNode,
    },

    Function {
        args: Vec<String>,
        r#type: Option<String>,
        body: BlockNode,
    },
}
