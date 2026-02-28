mod destructuring;
mod pattern;

use lux_ast::common::Span;
use oxc_ast::ast::{BindingPattern, Expression};

#[derive(Debug, Clone, Copy)]
pub(super) struct CollectedBinding<'a> {
    pub name: &'a str,
    pub span: Span,
}

pub(super) fn collect_pattern_bindings<'a>(
    pattern: &'a BindingPattern<'a>,
) -> Vec<CollectedBinding<'a>> {
    pattern::collect_pattern_bindings(pattern)
}

pub(super) fn collect_destructuring_expression_bindings<'a>(
    expression: &'a Expression<'a>,
) -> Vec<CollectedBinding<'a>> {
    destructuring::collect_destructuring_expression_bindings(expression)
}
