use super::namespace::NamespaceNode;

#[derive(Debug)]
pub struct ProjectNode {
    name: String,
    kind: ProjectKind,
    root: NamespaceNode,
}

impl ProjectNode {
    pub fn new<T: ToString>(name: T, kind: ProjectKind, root: NamespaceNode) -> Self {
        Self {
            name: name.to_string(),
            kind,
            root,
        }
    }
}

#[derive(Debug)]
pub enum ProjectKind {
    Package,
    Program,
}
