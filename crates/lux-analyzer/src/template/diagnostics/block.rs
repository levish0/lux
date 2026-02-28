use lux_ast::analysis::{AnalysisDiagnosticCode, AnalysisSeverity};
use lux_ast::template::root::{Fragment, FragmentNode};

use crate::template::context::TemplateAnalyzerContext;

pub(crate) fn warn_if_block_empty(
    fragment: &Fragment<'_>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    if fragment.nodes.len() != 1 {
        return;
    }

    let FragmentNode::Text(text) = &fragment.nodes[0] else {
        return;
    };

    if text.raw.trim().is_empty() {
        context.add_diagnostic(
            AnalysisSeverity::Warning,
            AnalysisDiagnosticCode::BlockEmpty,
            "Block contains only whitespace",
            text.span,
        );
    }
}
