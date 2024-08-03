use super::statements::StatementNode;

#[derive(Debug)]
pub struct BodyNode {
    pub kind: BodyKind,
}

#[derive(Debug)]
pub enum BodyKind {
    Inline(Box<BodyElementNode>),
    Multiline(Vec<BodyElementNode>),
}

#[derive(Debug)]
pub struct BodyElementNode {
    pub kind: BodyElementKind,
}

#[derive(Debug)]
pub enum BodyElementKind {
    Statement(StatementNode),
    Body(BodyNode),
    Empty,
}
