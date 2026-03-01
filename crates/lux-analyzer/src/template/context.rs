use lux_ast::analysis::{
    AnalysisDiagnostic, AnalysisDiagnosticCode, AnalysisSeverity, AnalysisTables,
    TemplateBindingAnalysis, TemplateBindingKind, TemplateReferenceAnalysis, TemplateScopeAnalysis,
    TemplateScopeId, TemplateScopeKind,
};
use lux_ast::common::Span;

pub(super) struct TemplateAnalyzerContext<'a> {
    tables: &'a mut AnalysisTables,
    scope_stack: Vec<TemplateScopeId>,
    nested_region_depth: u32,
    seen_svelte_head: bool,
    seen_svelte_body: bool,
    seen_svelte_window: bool,
    seen_svelte_document: bool,
    seen_svelte_options: bool,
}

impl<'a> TemplateAnalyzerContext<'a> {
    pub(super) fn new(tables: &'a mut AnalysisTables, root_span: Span) -> Self {
        let root_scope = push_scope_record(tables, TemplateScopeKind::Root, None, Some(root_span));
        Self {
            tables,
            scope_stack: vec![root_scope],
            nested_region_depth: 0,
            seen_svelte_head: false,
            seen_svelte_body: false,
            seen_svelte_window: false,
            seen_svelte_document: false,
            seen_svelte_options: false,
        }
    }

    pub(super) fn current_scope(&self) -> TemplateScopeId {
        *self
            .scope_stack
            .last()
            .expect("template analyzer scope stack should never be empty")
    }

    pub(super) fn create_child_scope(
        &mut self,
        kind: TemplateScopeKind,
        span: Option<Span>,
    ) -> TemplateScopeId {
        let parent = self.current_scope();
        push_scope_record(self.tables, kind, Some(parent), span)
    }

    pub(super) fn enter_scope(&mut self, scope: TemplateScopeId) {
        self.scope_stack.push(scope);
    }

    pub(super) fn exit_scope(&mut self) {
        self.scope_stack
            .pop()
            .expect("template analyzer attempted to pop empty scope stack");
    }

    pub(super) fn add_binding(
        &mut self,
        kind: TemplateBindingKind,
        name: &str,
        span: Option<Span>,
    ) {
        let scope = self.current_scope();
        self.add_binding_in_scope(scope, kind, name, span);
    }

    pub(super) fn add_binding_in_scope(
        &mut self,
        scope: TemplateScopeId,
        kind: TemplateBindingKind,
        name: &str,
        span: Option<Span>,
    ) {
        self.tables.template_bindings.push(TemplateBindingAnalysis {
            scope,
            kind,
            name: name.to_owned(),
            span,
        });
    }

    pub(super) fn add_reference(&mut self, name: &str, span: Span, is_read: bool, is_write: bool) {
        self.tables
            .template_references
            .push(TemplateReferenceAnalysis {
                scope: self.current_scope(),
                name: name.to_owned(),
                span,
                is_read,
                is_write,
            });
    }

    pub(super) fn has_binding_in_scope(
        &self,
        scope: TemplateScopeId,
        kind: TemplateBindingKind,
        name: &str,
    ) -> bool {
        self.tables
            .template_bindings
            .iter()
            .any(|binding| binding.scope == scope && binding.kind == kind && binding.name == name)
    }

    pub(super) fn add_diagnostic(
        &mut self,
        severity: AnalysisSeverity,
        code: AnalysisDiagnosticCode,
        message: impl Into<String>,
        span: Span,
    ) {
        self.tables.diagnostics.push(AnalysisDiagnostic {
            severity,
            code,
            message: message.into(),
            span,
        });
    }

    pub(super) fn with_nested_region<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.nested_region_depth += 1;
        let out = f(self);
        self.nested_region_depth -= 1;
        out
    }

    pub(super) fn is_inside_element_or_block(&self) -> bool {
        self.nested_region_depth > 0
    }

    pub(super) fn mark_svelte_head_seen(&mut self) -> bool {
        let already_seen = self.seen_svelte_head;
        self.seen_svelte_head = true;
        already_seen
    }

    pub(super) fn mark_svelte_body_seen(&mut self) -> bool {
        let already_seen = self.seen_svelte_body;
        self.seen_svelte_body = true;
        already_seen
    }

    pub(super) fn mark_svelte_window_seen(&mut self) -> bool {
        let already_seen = self.seen_svelte_window;
        self.seen_svelte_window = true;
        already_seen
    }

    pub(super) fn mark_svelte_document_seen(&mut self) -> bool {
        let already_seen = self.seen_svelte_document;
        self.seen_svelte_document = true;
        already_seen
    }

    pub(super) fn mark_svelte_options_seen(&mut self) -> bool {
        let already_seen = self.seen_svelte_options;
        self.seen_svelte_options = true;
        already_seen
    }
}

fn push_scope_record(
    tables: &mut AnalysisTables,
    kind: TemplateScopeKind,
    parent: Option<TemplateScopeId>,
    span: Option<Span>,
) -> TemplateScopeId {
    let id = TemplateScopeId(tables.template_scopes.len() as u32);
    tables.template_scopes.push(TemplateScopeAnalysis {
        id,
        kind,
        parent,
        span,
    });
    id
}
