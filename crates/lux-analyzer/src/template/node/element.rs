use lux_ast::analysis::{TemplateBindingKind, TemplateScopeKind};
use lux_ast::common::Span;
use lux_ast::template::attribute::AttributeNode;
use lux_ast::template::root::Fragment;

use super::super::binding::collect_destructuring_expression_bindings;
use super::super::context::TemplateAnalyzerContext;
use super::super::fragment;

pub(crate) fn analyze<'a>(
    span: Span,
    attributes: &[AttributeNode<'a>],
    children: &Fragment<'a>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    let element_scope = context.create_child_scope(TemplateScopeKind::Element, Some(span));

    for attribute in attributes {
        if let AttributeNode::LetDirective(let_directive) = attribute {
            if let Some(expression) = &let_directive.expression {
                for binding in collect_destructuring_expression_bindings(expression) {
                    context.add_binding_in_scope(
                        element_scope,
                        TemplateBindingKind::LetDirective,
                        binding.name,
                        Some(binding.span),
                    );
                }
            } else {
                context.add_binding_in_scope(
                    element_scope,
                    TemplateBindingKind::LetDirective,
                    let_directive.name,
                    Some(let_directive.span),
                );
            }
        }
    }

    context.enter_scope(element_scope);
    fragment::analyze_fragment(children, context);
    context.exit_scope();
}
