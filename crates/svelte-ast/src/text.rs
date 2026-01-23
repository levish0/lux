use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use crate::span::Span;

/*
 * interface Text extends BaseNode {
 *   type: 'Text';
 *   data: string;
 *   raw: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct Text {
    #[serde(flatten)]
    pub span: Span,
    pub data: String,
    pub raw: String,
}

/*
 * interface Comment extends BaseNode {
 *   type: 'Comment';
 *   data: string;
 * }
 */
#[derive(Debug, Clone, Serialize)]
pub struct Comment {
    #[serde(flatten)]
    pub span: Span,
    pub data: String,
}

/*
 * interface JSComment {
 *   type: 'Line' | 'Block';
 *   value: string;
 * }
 */
#[derive(Debug, Clone)]
pub struct JsComment {
    pub span: Option<Span>,
    pub kind: JsCommentKind,
    pub value: String,
}

impl Serialize for JsComment {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        let type_str = match self.kind {
            JsCommentKind::Line => "Line",
            JsCommentKind::Block => "Block",
        };
        map.serialize_entry("type", type_str)?;
        map.serialize_entry("value", &self.value)?;
        if let Some(span) = &self.span {
            map.serialize_entry("start", &span.start)?;
            map.serialize_entry("end", &span.end)?;
        }
        map.end()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsCommentKind {
    Line,
    Block,
}
