use lux_ast::analysis::{TemplateBindingKind, TemplateScopeKind};
use lux_ast::template::block::EachBlock;

use super::super::binding::collect_pattern_bindings;
use super::super::context::TemplateAnalyzerContext;
use super::super::diagnostics;
use super::super::fragment;
use super::super::reference;

pub(crate) fn analyze(block: &EachBlock<'_>, context: &mut TemplateAnalyzerContext<'_>) {
    diagnostics::validate_each_block(block, context);
    diagnostics::warn_if_block_empty(&block.body, context);
    if let Some(fallback) = &block.fallback {
        diagnostics::warn_if_block_empty(fallback, context);
    }

    reference::analyze_expression(&block.expression, context);
    if let Some(key_expression) = &block.key {
        reference::analyze_expression(key_expression, context);
    }

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
    context.with_nested_region(|context| {
        fragment::analyze_fragment(&block.body, context);
    });
    if let Some(fallback) = &block.fallback {
        context.with_nested_region(|context| {
            fragment::analyze_fragment(fallback, context);
        });
    }
    context.exit_scope();
}
