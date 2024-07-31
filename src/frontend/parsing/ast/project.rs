use super::namespace::NamespaceNode;

#[derive(Debug)]
pub struct ProjectNode {
    pub name: String,
    pub kind: ProjectKind,
    pub root: NamespaceNode,
}

#[derive(Debug)]
pub enum ProjectKind {
    Package,
    Program,
}
