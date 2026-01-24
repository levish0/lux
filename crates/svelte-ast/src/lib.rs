pub mod attributes;
pub mod blocks;
pub mod css;
pub mod elements;
pub mod node;
pub mod root;
pub mod span;
pub mod tags;
pub mod text;
pub mod utils;

/// Pre-serialized ESTree JSON node from OXC parser.
/// Stores the already-serialized ESTree representation so we don't need
/// to maintain typed AST nodes for JS/TS expressions.
#[derive(Debug, Clone)]
pub struct JsNode(pub serde_json::Value);

impl serde::Serialize for JsNode {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}
