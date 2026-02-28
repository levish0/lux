use lux_ast::analysis::{AnalysisDiagnosticCode, AnalysisSeverity};
use lux_ast::template::block::EachBlock;
use oxc_ast::ast::BindingPattern;
use oxc_span::GetSpan;

use crate::template::context::TemplateAnalyzerContext;

pub(crate) fn validate_each_block(
    block: &EachBlock<'_>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    if let Some(key_expression) = &block.key {
        let is_index_key = matches!(
            key_expression,
            oxc_ast::ast::Expression::Identifier(identifier)
                if block.index.is_some() && Some(identifier.name.as_str()) == block.index
        );

        let is_keyed = !is_index_key;
        if is_keyed && block.context.is_none() {
            context.add_diagnostic(
                AnalysisSeverity::Error,
                AnalysisDiagnosticCode::EachKeyWithoutContext,
                "Keyed each block requires an `as` context",
                key_expression.span(),
            );
        }
    }

    if let Some(pattern) = &block.context {
        if let BindingPattern::BindingIdentifier(identifier) = pattern {
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
