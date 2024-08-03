use std::fmt::Debug;

use super::items::ItemNode;

#[derive(Debug)]
pub struct NamespaceNode {
    pub kind: NamespaceKind,
    pub items: Vec<ItemNode>,
}

impl NamespaceNode {
    pub fn new(kind: NamespaceKind, items: Vec<ItemNode>) -> Self {
        Self { kind, items }
    }
}

#[derive(Debug)]
pub enum NamespaceKind {
    Root,
    Sub(NamespacePath),
}

pub struct NamespacePath {
    names: Vec<String>,
}

impl NamespacePath {
    pub fn new(names: Vec<String>) -> Self {
        Self { names }
    }

    pub fn name(&self) -> String {
        self.names.last().unwrap().clone()
    }

    pub fn parent(&self) -> Option<NamespacePath> {
        if self.names.len() == 1 {
            None
        } else {
            Some(NamespacePath::new(
                self.names[0..self.names.len() - 1].to_vec(),
            ))
        }
    }
}

impl Debug for NamespacePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl ToString for NamespacePath {
    fn to_string(&self) -> String {
        self.names.join("::")
    }
}
