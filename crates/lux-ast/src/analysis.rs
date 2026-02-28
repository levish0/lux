use std::collections::HashMap;

use oxc_span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalysisNodeKind {
    CssRule,
    ComplexSelector,
    RelativeSelector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpanKey {
    pub kind: AnalysisNodeKind,
    pub start: u32,
    pub end: u32,
}

impl SpanKey {
    pub fn new(kind: AnalysisNodeKind, span: Span) -> Self {
        Self {
            kind,
            start: span.start,
            end: span.end,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CssRuleAnalysis {
    pub parent_rule: Option<SpanKey>,
    pub has_local_selectors: bool,
    pub has_global_selectors: bool,
    pub is_global_block: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ComplexSelectorAnalysis {
    pub is_global: bool,
    pub used: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RelativeSelectorAnalysis {
    pub is_global: bool,
    pub is_global_like: bool,
    pub scoped: bool,
}

#[derive(Debug, Default)]
pub struct AnalysisTables {
    pub css_rules: HashMap<SpanKey, CssRuleAnalysis>,
    pub complex_selectors: HashMap<SpanKey, ComplexSelectorAnalysis>,
    pub relative_selectors: HashMap<SpanKey, RelativeSelectorAnalysis>,
}
