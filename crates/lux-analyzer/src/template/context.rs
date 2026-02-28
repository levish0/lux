use lux_ast::analysis::{
    AnalysisTables, TemplateBindingAnalysis, TemplateBindingKind, TemplateScopeAnalysis,
    TemplateScopeId, TemplateScopeKind,
};
use lux_ast::common::Span;

pub(super) struct TemplateAnalyzerContext<'a> {
    tables: &'a mut AnalysisTables,
    scope_stack: Vec<TemplateScopeId>,
}

impl<'a> TemplateAnalyzerContext<'a> {
    pub(super) fn new(tables: &'a mut AnalysisTables, root_span: Span) -> Self {
        let root_scope = push_scope_record(tables, TemplateScopeKind::Root, None, Some(root_span));
        Self {
            tables,
            scope_stack: vec![root_scope],
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
