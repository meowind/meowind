use super::namespaces::{NamespaceKind, NamespaceNode};

#[derive(Debug)]
pub struct ProjectNode {
    pub name: String,
    pub kind: ProjectKind,
    pub root: NamespaceNode,
}

impl Default for ProjectNode {
    fn default() -> Self {
        Self {
            name: String::new(),
            kind: ProjectKind::Program,
            root: NamespaceNode::new(NamespaceKind::Root, Vec::new()),
        }
    }
}

#[derive(Debug)]
pub enum ProjectKind {
    Package,
    Program,
}
