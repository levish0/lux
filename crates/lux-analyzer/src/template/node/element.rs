use lux_ast::analysis::{TemplateBindingKind, TemplateScopeKind};
use lux_ast::common::Span;
use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::root::Fragment;
use lux_ast::template::tag::TextOrExpressionTag;

use super::super::binding::collect_destructuring_expression_bindings;
use super::super::context::TemplateAnalyzerContext;
use super::super::fragment;
use super::super::reference;

pub(crate) fn analyze<'a>(
    span: Span,
    attributes: &[AttributeNode<'a>],
    children: &Fragment<'a>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    let element_scope = context.create_child_scope(TemplateScopeKind::Element, Some(span));

    for attribute in attributes {
        match attribute {
            AttributeNode::Attribute(attribute) => {
                analyze_attribute_value(&attribute.value, context);
            }
            AttributeNode::SpreadAttribute(attribute) => {
                reference::analyze_expression(&attribute.expression, context);
            }
            AttributeNode::BindDirective(directive) => {
                reference::analyze_expression(&directive.expression, context);
            }
            AttributeNode::ClassDirective(directive) => {
                reference::analyze_expression(&directive.expression, context);
            }
            AttributeNode::StyleDirective(directive) => match &directive.value {
                lux_ast::template::directive::StyleDirectiveValue::True => {}
                lux_ast::template::directive::StyleDirectiveValue::ExpressionTag(tag) => {
                    reference::analyze_expression(&tag.expression, context);
                }
                lux_ast::template::directive::StyleDirectiveValue::Sequence(sequence) => {
                    for chunk in sequence {
                        if let TextOrExpressionTag::ExpressionTag(tag) = chunk {
                            reference::analyze_expression(&tag.expression, context);
                        }
                    }
                }
            },
            AttributeNode::OnDirective(directive) => {
                if let Some(expression) = &directive.expression {
                    reference::analyze_expression(expression, context);
                }
            }
            AttributeNode::TransitionDirective(directive) => {
                if let Some(expression) = &directive.expression {
                    reference::analyze_expression(expression, context);
                }
            }
            AttributeNode::AnimateDirective(directive) => {
                if let Some(expression) = &directive.expression {
                    reference::analyze_expression(expression, context);
                }
            }
            AttributeNode::UseDirective(directive) => {
                if let Some(expression) = &directive.expression {
                    reference::analyze_expression(expression, context);
                }
            }
            AttributeNode::LetDirective(let_directive) => {
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
            AttributeNode::AttachTag(tag) => {
                reference::analyze_expression(&tag.expression, context);
            }
        }
    }

    context.enter_scope(element_scope);
    fragment::analyze_fragment(children, context);
    context.exit_scope();
}

fn analyze_attribute_value(value: &AttributeValue<'_>, context: &mut TemplateAnalyzerContext<'_>) {
    match value {
        AttributeValue::True => {}
        AttributeValue::ExpressionTag(tag) => {
            reference::analyze_expression(&tag.expression, context);
        }
        AttributeValue::Sequence(sequence) => {
            for chunk in sequence {
                if let TextOrExpressionTag::ExpressionTag(tag) = chunk {
                    reference::analyze_expression(&tag.expression, context);
                }
            }
        }
    }
}
