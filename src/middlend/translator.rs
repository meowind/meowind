use crate::frontend::parsing::ast::project::ProjectNode;

pub struct Translator {
    ast: ProjectNode,
}

impl Translator {
    pub fn new(ast: ProjectNode) -> Self {
        Self {
            ast,
            ..Default::default()
        }
    }

    pub fn translate(ast: ProjectNode) -> Self {
        let translator = Self::new(ast);

        translator
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self {
            ast: ProjectNode::default(),
        }
    }
}
