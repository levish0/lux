use std::collections::HashMap;

use oxc_span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalysisNodeKind {
    CssRule,
    ComplexSelector,
    RelativeSelector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemplateScopeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateScopeKind {
    Root,
    Element,
    Each,
    AwaitThen,
    AwaitCatch,
    Snippet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateBindingKind {
    EachContext,
    EachIndex,
    AwaitValue,
    AwaitError,
    SnippetName,
    SnippetParameter,
    LetDirective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalysisSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalysisDiagnosticCode {
    EachKeyWithoutContext,
    EachInvalidContextIdentifier,
    BlockEmpty,
    SnippetDuplicateName,
    SnippetInvalidRestParameter,
    SnippetShadowingProp,
    SnippetChildrenConflict,
    TemplateAssignmentToConst,
    TemplateAssignmentToImport,
    TemplateAssignmentToBinding,
    BindDirectiveInvalidExpression,
    BindDirectiveUnknownName,
    BindDirectiveInvalidTarget,
    BindDirectiveGroupInvalidExpression,
    BindDirectiveInputTypeInvalid,
    BindDirectiveInputTypeMismatch,
    BindDirectiveSelectMultipleDynamic,
    BindDirectiveContenteditableMissing,
    BindDirectiveContenteditableDynamic,
    LetDirectiveInvalidPlacement,
    RenderTagInvalidSpreadArgument,
    RenderTagInvalidCallExpression,
    ScriptRuneInvalidArgumentsLength,
    ScriptRuneInvalidArguments,
    TemplateRuneInvalidPlacement,
    SvelteMetaInvalidPlacement,
    SvelteMetaInvalidContent,
    SvelteMetaDuplicate,
}

#[derive(Debug, Clone)]
pub struct AnalysisDiagnostic {
    pub severity: AnalysisSeverity,
    pub code: AnalysisDiagnosticCode,
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptTarget {
    Instance,
    Module,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptRuneKind {
    Known,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ScriptScopeAnalysis {
    pub target: ScriptTarget,
    pub id: u32,
    pub parent: Option<u32>,
    pub flags: u16,
    pub node_id: u32,
}

#[derive(Debug, Clone)]
pub struct ScriptSymbolAnalysis {
    pub target: ScriptTarget,
    pub id: u32,
    pub name: String,
    pub scope_id: u32,
    pub declaration_node_id: u32,
    pub declaration_span: Span,
    pub flags: u32,
    pub mutated: bool,
    pub unused: bool,
}

#[derive(Debug, Clone)]
pub struct ScriptReferenceAnalysis {
    pub target: ScriptTarget,
    pub id: u32,
    pub name: String,
    pub span: Span,
    pub scope_id: u32,
    pub symbol_id: Option<u32>,
    pub is_read: bool,
    pub is_write: bool,
}

#[derive(Debug, Clone)]
pub struct ScriptRuneAnalysis {
    pub target: ScriptTarget,
    pub name: String,
    pub kind: ScriptRuneKind,
    pub span: Span,
    pub callee_span: Span,
    pub argument_count: u32,
    pub is_state_creation: bool,
}

#[derive(Debug, Clone)]
pub struct ScriptImportAnalysis {
    pub target: ScriptTarget,
    pub span: Span,
    pub source: String,
    pub local_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TemplateScopeAnalysis {
    pub id: TemplateScopeId,
    pub kind: TemplateScopeKind,
    pub parent: Option<TemplateScopeId>,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct TemplateBindingAnalysis {
    pub scope: TemplateScopeId,
    pub kind: TemplateBindingKind,
    pub name: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct TemplateReferenceAnalysis {
    pub scope: TemplateScopeId,
    pub name: String,
    pub span: Span,
    pub is_read: bool,
    pub is_write: bool,
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
    pub script_scopes: Vec<ScriptScopeAnalysis>,
    pub script_symbols: Vec<ScriptSymbolAnalysis>,
    pub script_references: Vec<ScriptReferenceAnalysis>,
    pub script_runes: Vec<ScriptRuneAnalysis>,
    pub script_imports: Vec<ScriptImportAnalysis>,
    pub template_scopes: Vec<TemplateScopeAnalysis>,
    pub template_bindings: Vec<TemplateBindingAnalysis>,
    pub template_references: Vec<TemplateReferenceAnalysis>,
    pub diagnostics: Vec<AnalysisDiagnostic>,
}
