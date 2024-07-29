use super::{expressions::ExpressionNode, statements::StatementNode};

#[derive(Debug)]
pub struct BlockNode {
    elements: Vec<BlockElementKind>,
}

#[derive(Debug)]
pub enum BlockElementKind {
    Statement(StatementNode),
    Expression(ExpressionNode),
    Block(BlockNode),
}
