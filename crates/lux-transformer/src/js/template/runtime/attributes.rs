use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::directive::{StyleDirective, StyleDirectiveValue, StyleModifier};
use lux_ast::template::tag::TextOrExpressionTag;
use lux_utils::attributes::is_boolean_attribute;
use oxc_allocator::CloneIn;
use oxc_ast::{
    AstBuilder, NONE,
    ast::{BinaryOperator, Expression, LogicalOperator},
};
use oxc_span::SPAN;

use super::expr::{
    escape_attr_expression, join_chunks_expression, string_expr, stringify_expression,
};
use super::scope::{RuntimeScope, is_valid_js_identifier, resolve_expression};

pub(super) fn render_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    attribute: &AttributeNode<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    match attribute {
        AttributeNode::Attribute(attribute) => {
            if is_event_attribute_name(attribute.name) {
                return string_expr(ast, "");
            }
            match &attribute.value {
                AttributeValue::True => string_expr(ast, &format!(" {}", attribute.name)),
                AttributeValue::ExpressionTag(tag) => render_named_expression_attribute(
                    ast,
                    attribute.name,
                    resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
                ),
                AttributeValue::Sequence(chunks) => {
                    let mut value_parts = ast.vec();
                    for chunk in chunks {
                        let chunk_expr = match chunk {
                            TextOrExpressionTag::Text(text) => string_expr(ast, text.raw),
                            TextOrExpressionTag::ExpressionTag(tag) => stringify_expression(
                                ast,
                                resolve_expression(
                                    ast,
                                    tag.expression.clone_in(ast.allocator),
                                    scope,
                                ),
                            ),
                        };
                        value_parts.push(chunk_expr);
                    }

                    render_named_expression_attribute(
                        ast,
                        attribute.name,
                        join_chunks_expression(ast, value_parts),
                    )
                }
            }
        }
        AttributeNode::SpreadAttribute(attribute) => render_spread_attribute_expression(
            ast,
            resolve_expression(ast, attribute.expression.clone_in(ast.allocator), scope),
        ),
        AttributeNode::BindDirective(attribute) => {
            if attribute.name == "this" {
                string_expr(ast, "")
            } else {
                render_named_expression_attribute(
                    ast,
                    attribute.name,
                    resolve_expression(ast, attribute.expression.clone_in(ast.allocator), scope),
                )
            }
        }
        AttributeNode::ClassDirective(attribute) => render_class_directive_attribute_expression(
            ast,
            attribute.name,
            resolve_expression(ast, attribute.expression.clone_in(ast.allocator), scope),
        ),
        AttributeNode::StyleDirective(attribute) => {
            render_style_directive_attribute_expression(ast, attribute, scope)
        }
        AttributeNode::OnDirective(_)
        | AttributeNode::TransitionDirective(_)
        | AttributeNode::AnimateDirective(_)
        | AttributeNode::UseDirective(_)
        | AttributeNode::LetDirective(_)
        | AttributeNode::AttachTag(_) => string_expr(ast, ""),
    }
}

fn render_named_expression_attribute<'a>(
    ast: AstBuilder<'a>,
    name: &str,
    value: Expression<'a>,
) -> Expression<'a> {
    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_attr")),
        NONE,
        ast.vec_from_array([
            string_expr(ast, name).into(),
            value.into(),
            ast.expression_boolean_literal(SPAN, is_boolean_attribute(name))
                .into(),
        ]),
        false,
    )
}

fn render_class_directive_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    name: &str,
    value: Expression<'a>,
) -> Expression<'a> {
    let class_attr = join_chunks_expression(
        ast,
        ast.vec_from_array([
            string_expr(ast, " class=\""),
            string_expr(ast, name),
            string_expr(ast, "\""),
        ]),
    );
    ast.expression_conditional(SPAN, value, class_attr, string_expr(ast, ""))
}

fn render_style_directive_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    directive: &StyleDirective<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let value = render_style_directive_value_expression(ast, directive, scope);
    let style_body = if directive.modifiers.contains(&StyleModifier::Important) {
        join_chunks_expression(
            ast,
            ast.vec_from_array([
                value.clone_in(ast.allocator),
                string_expr(ast, " !important"),
            ]),
        )
    } else {
        value.clone_in(ast.allocator)
    };
    let style_attr = join_chunks_expression(
        ast,
        ast.vec_from_array([
            string_expr(ast, &format!(" style=\"{}: ", directive.name)),
            escape_attr_expression(ast, stringify_expression(ast, style_body)),
            string_expr(ast, ";\""),
        ]),
    );

    let omit = is_falsy_attribute_value_expression(ast, value);
    ast.expression_conditional(SPAN, omit, string_expr(ast, ""), style_attr)
}

fn render_style_directive_value_expression<'a>(
    ast: AstBuilder<'a>,
    directive: &StyleDirective<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    match &directive.value {
        StyleDirectiveValue::True => {
            if is_valid_js_identifier(directive.name) {
                resolve_expression(
                    ast,
                    ast.expression_identifier(SPAN, ast.ident(directive.name)),
                    scope,
                )
            } else {
                string_expr(ast, "")
            }
        }
        StyleDirectiveValue::ExpressionTag(tag) => {
            resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope)
        }
        StyleDirectiveValue::Sequence(chunks) => {
            let mut parts = ast.vec();
            for chunk in chunks {
                let chunk_expression = match chunk {
                    TextOrExpressionTag::Text(text) => string_expr(ast, text.raw),
                    TextOrExpressionTag::ExpressionTag(tag) => stringify_expression(
                        ast,
                        resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
                    ),
                };
                parts.push(chunk_expression);
            }
            join_chunks_expression(ast, parts)
        }
    }
}

fn render_spread_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    spread_expression: Expression<'a>,
) -> Expression<'a> {
    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("__lux_attributes")),
        NONE,
        ast.vec1(spread_expression.into()),
        false,
    )
}

fn is_falsy_attribute_value_expression<'a>(
    ast: AstBuilder<'a>,
    value: Expression<'a>,
) -> Expression<'a> {
    let is_nullish = ast.expression_binary(
        SPAN,
        value.clone_in(ast.allocator),
        BinaryOperator::Equality,
        ast.expression_null_literal(SPAN),
    );
    let is_false = ast.expression_binary(
        SPAN,
        value,
        BinaryOperator::StrictEquality,
        ast.expression_boolean_literal(SPAN, false),
    );
    ast.expression_logical(SPAN, is_nullish, LogicalOperator::Or, is_false)
}

fn is_event_attribute_name(name: &str) -> bool {
    name.len() > 2 && name.starts_with("on")
}
