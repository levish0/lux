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
    AttributeEmptyShorthand,
    ExpectedExpression,
    ExpectedAttributeValue,
    JsParseError,
    CssSelectorInvalid,
    SvelteMetaDuplicate,
    RenderTagInvalidExpression,
    ConstTagInvalidExpression,
    SvelteMetaInvalidTag,
    SvelteMetaInvalidPlacement,
    TagInvalidName,
    VoidElementInvalidContent,
    ElementInvalidClosingTag,
    DirectiveMissingName,
    DirectiveInvalidValue,
    BlockInvalidPlacement,
    TagInvalidPlacement,
    ScriptDuplicate,
    StyleDuplicate,
    SvelteElementMissingThis,
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
