use lux_ast::analysis::{AnalysisDiagnosticCode, AnalysisSeverity};
use lux_ast::template::tag::RenderTag;
use oxc_ast::ast::{Argument, Expression};

use crate::template::context::TemplateAnalyzerContext;

pub(crate) fn validate_render_tag(tag: &RenderTag<'_>, context: &mut TemplateAnalyzerContext<'_>) {
    let Expression::CallExpression(call_expression) = &tag.expression else {
        return;
    };

    if call_expression
        .arguments
        .iter()
        .any(|argument| matches!(argument, Argument::SpreadElement(_)))
    {
        context.add_diagnostic(
            AnalysisSeverity::Error,
            AnalysisDiagnosticCode::RenderTagInvalidSpreadArgument,
            "render tag arguments cannot contain spread elements",
            tag.span,
        );
    }

    if is_forbidden_member_call(&call_expression.callee) {
        context.add_diagnostic(
            AnalysisSeverity::Error,
            AnalysisDiagnosticCode::RenderTagInvalidCallExpression,
            "render tag cannot call `.bind`, `.apply`, or `.call`",
            tag.span,
        );
    }
}

fn is_forbidden_member_call(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::StaticMemberExpression(member) => {
            matches!(member.property.name.as_str(), "bind" | "apply" | "call")
        }
        Expression::ComputedMemberExpression(member) => {
            if let Expression::StringLiteral(literal) = &member.expression {
                matches!(literal.value.as_str(), "bind" | "apply" | "call")
            } else {
                false
            }
        }
        _ => false,
    }
}
