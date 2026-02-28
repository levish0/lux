use lux_ast::analysis::{AnalysisDiagnosticCode, AnalysisSeverity};
use lux_ast::template::block::EachBlock;
use oxc_span::GetSpan;

use crate::template::context::TemplateAnalyzerContext;

pub(crate) fn validate_each_block(
    block: &EachBlock<'_>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    if block.key.is_some() && block.context.is_none() {
        if let Some(key_expression) = &block.key {
            context.add_diagnostic(
                AnalysisSeverity::Error,
                AnalysisDiagnosticCode::EachKeyWithoutContext,
                "Keyed each block requires an `as` context",
                key_expression.span(),
            );
        }
    }

    if let Some(pattern) = &block.context {
        for identifier in pattern.get_binding_identifiers() {
            let name = identifier.name.as_str();
            if name == "$state" || name == "$derived" {
                context.add_diagnostic(
                    AnalysisSeverity::Error,
                    AnalysisDiagnosticCode::EachInvalidContextIdentifier,
                    format!("Invalid each context identifier `{name}`"),
                    identifier.span,
                );
            }
        }
    }
}
