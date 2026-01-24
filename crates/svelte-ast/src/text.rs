use crate::span::Span;

/*
 * interface Text extends BaseNode {
 *   type: 'Text';
 *   data: string;
 *   raw: string;
 * }
 */
#[derive(Debug, Clone)]
pub struct Text {
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
#[derive(Debug, Clone)]
pub struct Comment {
    pub span: Span,
    pub data: String,
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
pub struct JsComment {
    pub span: Span,
    pub kind: JsCommentKind,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsCommentKind {
    Line,
    Block,
}
