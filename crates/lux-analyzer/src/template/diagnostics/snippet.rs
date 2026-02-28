use lux_ast::analysis::{AnalysisDiagnosticCode, AnalysisSeverity, TemplateBindingKind};
use lux_ast::template::block::SnippetBlock;

use crate::template::context::TemplateAnalyzerContext;

pub(super) fn validate_snippet_block(
    block: &SnippetBlock<'_>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    let scope = context.current_scope();
    let name = block.expression.name.as_str();

    if context.has_binding_in_scope(scope, TemplateBindingKind::SnippetName, name) {
        context.add_diagnostic(
            AnalysisSeverity::Error,
            AnalysisDiagnosticCode::SnippetDuplicateName,
            format!("Duplicate snippet name `{name}`"),
            block.expression.span,
        );
    }
}
