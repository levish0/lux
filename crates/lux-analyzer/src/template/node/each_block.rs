use lux_ast::analysis::{TemplateBindingKind, TemplateScopeKind};
use lux_ast::template::block::EachBlock;

use super::super::binding::collect_pattern_bindings;
use super::super::context::TemplateAnalyzerContext;
use super::super::fragment;

pub(crate) fn analyze(block: &EachBlock<'_>, context: &mut TemplateAnalyzerContext<'_>) {
    let each_scope = context.create_child_scope(TemplateScopeKind::Each, Some(block.span));

    if let Some(pattern) = &block.context {
        for binding in collect_pattern_bindings(pattern) {
            context.add_binding_in_scope(
                each_scope,
                TemplateBindingKind::EachContext,
                binding.name,
                Some(binding.span),
            );
        }
    }

    if let Some(index) = block.index {
        context.add_binding_in_scope(each_scope, TemplateBindingKind::EachIndex, index, None);
    }

    context.enter_scope(each_scope);
    fragment::analyze_fragment(&block.body, context);
    if let Some(fallback) = &block.fallback {
        fragment::analyze_fragment(fallback, context);
    }
    context.exit_scope();
}
