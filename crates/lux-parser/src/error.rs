use oxc_span::Span;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ErrorKind,
    pub code: Option<&'static str>,
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ParseWarning {
    pub kind: WarningKind,
    pub code: &'static str,
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    UnexpectedEof,
    UnexpectedToken,
    UnclosedElement,
    UnclosedBlock,
    UnclosedComment,
    UnclosedString,
    InvalidEntity,
    InvalidExpression,
    InvalidAttribute,
    InvalidDirective,
    InvalidTagName,
    DuplicateAttribute,
    MissingAttribute,
    InvalidCss,
    InvalidScript,
    InvalidSvelteOptions,
    General,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningKind {
    InvalidScript,
    General,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ParseError {
    pub fn new(kind: ErrorKind, span: Span, message: impl Into<String>) -> Self {
        Self {
            kind,
            code: None,
            span,
            message: message.into(),
        }
    }

    pub fn with_code(
        kind: ErrorKind,
        code: &'static str,
        span: Span,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            code: Some(code),
            span,
            message: message.into(),
        }
    }

    pub fn unexpected_eof(offset: u32) -> Self {
        Self::new(
            ErrorKind::UnexpectedEof,
            Span::new(offset, offset),
            "Unexpected end of input",
        )
    }

    pub fn unclosed_element(name: &str, span: Span) -> Self {
        Self::new(
            ErrorKind::UnclosedElement,
            span,
            format!("'<{name}>' was left open"),
        )
    }

    pub fn unclosed_block(name: &str, span: Span) -> Self {
        Self::new(
            ErrorKind::UnclosedBlock,
            span,
            format!("Block was left open: {{#{name}}}"),
        )
    }
}

impl ParseWarning {
    pub fn new(
        kind: WarningKind,
        code: &'static str,
        span: Span,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            code,
            span,
            message: message.into(),
        }
    }
}
