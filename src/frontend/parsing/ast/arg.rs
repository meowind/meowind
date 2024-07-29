use super::{expressions::ExpressionKind, r#type::TypeNode};

#[derive(Debug)]
pub struct ArgumentNode {
    name: String,
    r#type: TypeNode,
    default: Option<ExpressionKind>,
}
