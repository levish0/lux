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
