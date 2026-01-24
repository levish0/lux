use serde::Serialize;
use serde::ser::SerializeMap;

use crate::span::Span;

/*
 * interface Text extends BaseNode {
 *   type: 'Text';
 *   data: string;
 *   raw: string;
 * }
 */
#[derive(Debug, Clone)]
pub struct Text<'a> {
    pub span: Span,
    pub data: &'a str,
    pub raw: &'a str,
}

impl Serialize for Text<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "Text")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("raw", &self.raw)?;
        map.serialize_entry("data", &self.data)?;
        map.end()
    }
}

/*
 * interface Comment extends BaseNode {
 *   type: 'Comment';
 *   data: string;
 * }
 */
#[derive(Debug, Clone)]
pub struct Comment<'a> {
    pub span: Span,
    pub data: &'a str,
}

impl Serialize for Comment<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry("type", "Comment")?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("data", &self.data)?;
        map.end()
    }
}

/*
 * interface JSComment {
 *   type: 'Line' | 'Block';
 *   value: string;
 *   start: number;
 *   end: number;
 * }
 */
#[derive(Debug, Clone)]
pub struct JsComment<'a> {
    pub span: Span,
    pub kind: JsCommentKind,
    pub value: &'a str,
}

impl Serialize for JsComment<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut map = s.serialize_map(None)?;
        map.serialize_entry(
            "type",
            match self.kind {
                JsCommentKind::Line => "Line",
                JsCommentKind::Block => "Block",
            },
        )?;
        map.serialize_entry("start", &self.span.start)?;
        map.serialize_entry("end", &self.span.end)?;
        map.serialize_entry("value", &self.value)?;
        map.end()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsCommentKind {
    Line,
    Block,
}
