use super::statements::StatementNode;

#[derive(Debug)]
pub struct BlockNode {
    pub kind: BlockKind,
}

#[derive(Debug)]
pub enum BlockKind {
    Inline(Box<BlockElementNode>),
    Multiline(Vec<BlockElementNode>),
}

#[derive(Debug)]
pub struct BlockElementNode {
    pub kind: BlockElementKind,
}

#[derive(Debug)]
pub enum BlockElementKind {
    Statement(StatementNode),
    Block(BlockNode),
    Empty,
}
