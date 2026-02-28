use lux_ast::analysis::{TemplateBindingKind, TemplateScopeKind};
use lux_ast::template::block::SnippetBlock;

use super::super::binding::collect_pattern_bindings;
use super::super::context::TemplateAnalyzerContext;
use super::super::fragment;

pub(crate) fn analyze(block: &SnippetBlock<'_>, context: &mut TemplateAnalyzerContext<'_>) {
    context.add_binding(
        TemplateBindingKind::SnippetName,
        block.expression.name.as_str(),
        Some(block.expression.span),
    );

    let snippet_scope = context.create_child_scope(TemplateScopeKind::Snippet, Some(block.span));

    for parameter in &block.parameters {
        for binding in collect_pattern_bindings(parameter) {
            context.add_binding_in_scope(
                snippet_scope,
                TemplateBindingKind::SnippetParameter,
                binding.name,
                Some(binding.span),
            );
        }
    }

    context.enter_scope(snippet_scope);
    fragment::analyze_fragment(&block.body, context);
    context.exit_scope();
}
