use lux_ast::analysis::{AnalysisDiagnosticCode, AnalysisSeverity};
use lux_ast::template::directive::BindDirective;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use crate::template::context::TemplateAnalyzerContext;

pub(crate) fn validate_bind_directive_expression(
    directive: &BindDirective<'_>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    if is_valid_bind_expression(&directive.expression) {
        return;
    }

    context.add_diagnostic(
        AnalysisSeverity::Error,
        AnalysisDiagnosticCode::BindDirectiveInvalidExpression,
        "bind directive expects an assignable expression or getter/setter pair",
        directive.expression.span(),
    );
}

fn is_valid_bind_expression(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => true,
        Expression::SequenceExpression(sequence) => sequence.expressions.len() == 2,
        Expression::ParenthesizedExpression(parenthesized) => {
            is_valid_bind_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(expression) => is_valid_bind_expression(&expression.expression),
        Expression::TSSatisfiesExpression(expression) => {
            is_valid_bind_expression(&expression.expression)
        }
        Expression::TSNonNullExpression(expression) => {
            is_valid_bind_expression(&expression.expression)
        }
        Expression::TSTypeAssertion(expression) => is_valid_bind_expression(&expression.expression),
        _ => false,
    }
}
