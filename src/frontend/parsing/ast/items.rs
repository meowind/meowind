use super::{block::BlockNode, expressions::ExpressionNode, r#type::TypeNode};

#[derive(Debug)]
pub struct ItemNode {
    name: String,
    public: bool,
    kind: ItemKind,
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
