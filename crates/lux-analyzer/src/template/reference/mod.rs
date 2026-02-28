mod collector;
mod mode;

use oxc_ast::ast::Expression;
use oxc_ast_visit::Visit;

use super::context::TemplateAnalyzerContext;
use collector::ExpressionReferenceCollector;

pub(super) fn analyze_expression(
    expression: &Expression<'_>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    let mut collector = ExpressionReferenceCollector::new(context);
    collector.visit_expression(expression);
}

pub(super) fn analyze_bind_expression(
    expression: &Expression<'_>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    analyze_expression(expression, context);

    if let Some(identifier) = extract_bind_target_identifier(expression) {
        context.add_reference(identifier.name.as_str(), identifier.span, true, true);
    }
}

fn extract_bind_target_identifier<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a oxc_ast::ast::IdentifierReference<'a>> {
    let mut current = expression;

    loop {
        current = match current {
            Expression::ParenthesizedExpression(expression) => &expression.expression,
            Expression::TSAsExpression(expression) => &expression.expression,
            Expression::TSSatisfiesExpression(expression) => &expression.expression,
            Expression::TSNonNullExpression(expression) => &expression.expression,
            Expression::TSTypeAssertion(expression) => &expression.expression,
            _ => break,
        };
    }

    match current {
        Expression::Identifier(identifier) => Some(identifier),
        _ => None,
    }
}
