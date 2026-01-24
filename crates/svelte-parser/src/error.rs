use svelte_ast::span::Span;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ErrorKind,
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    UnexpectedEof,
    ExpectedToken,
    ElementUnclosed,
    BlockUnclosed,
    BlockUnexpectedClose,
    ExpectedBlockType,
    ExpectedTag,
    AttributeDuplicate,
    JsParseError,
    CssSelectorInvalid,
    SvelteMetaDuplicate,
    RenderTagInvalidExpression,
    ConstTagInvalidExpression,
    SvelteMetaInvalidTag,
    SvelteMetaInvalidPlacement,
}

impl ParseError {
    pub fn new(kind: ErrorKind, span: Span, message: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            message: message.into(),
        }
    }
}
