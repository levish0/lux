use lux_ast::analysis::{AnalysisDiagnosticCode, AnalysisSeverity};
use lux_ast::template::directive::LetDirective;

use crate::template::context::TemplateAnalyzerContext;

pub(crate) fn report_invalid_let_directive_placement(
    directive: &LetDirective<'_>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    context.add_diagnostic(
        AnalysisSeverity::Error,
        AnalysisDiagnosticCode::LetDirectiveInvalidPlacement,
        "let directive is not valid on this element type",
        directive.span,
    );
}
