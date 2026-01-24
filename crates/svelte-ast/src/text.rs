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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsCommentKind {
    Line,
    Block,
}
