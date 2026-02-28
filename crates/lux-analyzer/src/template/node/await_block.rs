use lux_ast::analysis::{TemplateBindingKind, TemplateScopeKind};
use lux_ast::template::block::AwaitBlock;

use super::super::binding::collect_pattern_bindings;
use super::super::context::TemplateAnalyzerContext;
use super::super::diagnostics;
use super::super::fragment;
use super::super::reference;

pub(crate) fn analyze(block: &AwaitBlock<'_>, context: &mut TemplateAnalyzerContext<'_>) {
    if let Some(pending) = &block.pending {
        diagnostics::warn_if_block_empty(pending, context);
    }
    if let Some(then_fragment) = &block.then {
        diagnostics::warn_if_block_empty(then_fragment, context);
    }
    if let Some(catch_fragment) = &block.catch {
        diagnostics::warn_if_block_empty(catch_fragment, context);
    }

    reference::analyze_expression(&block.expression, context);

    if let Some(pending) = &block.pending {
        fragment::analyze_fragment(pending, context);
    }

    if let Some(then_fragment) = &block.then {
        let then_scope = context.create_child_scope(TemplateScopeKind::AwaitThen, Some(block.span));
        if let Some(value_pattern) = &block.value {
            for binding in collect_pattern_bindings(value_pattern) {
                context.add_binding_in_scope(
                    then_scope,
                    TemplateBindingKind::AwaitValue,
                    binding.name,
                    Some(binding.span),
                );
            }
        }

        context.enter_scope(then_scope);
        fragment::analyze_fragment(then_fragment, context);
        context.exit_scope();
    }

    if let Some(catch_fragment) = &block.catch {
        let catch_scope =
            context.create_child_scope(TemplateScopeKind::AwaitCatch, Some(block.span));
        if let Some(error_pattern) = &block.error {
            for binding in collect_pattern_bindings(error_pattern) {
                context.add_binding_in_scope(
                    catch_scope,
                    TemplateBindingKind::AwaitError,
                    binding.name,
                    Some(binding.span),
                );
            }
        }

        context.enter_scope(catch_scope);
        fragment::analyze_fragment(catch_fragment, context);
        context.exit_scope();
    }
}
