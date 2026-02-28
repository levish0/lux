mod binding;
mod context;
mod diagnostics;
mod fragment;
mod node;
mod reference;

use lux_ast::analysis::AnalysisTables;
use lux_ast::template::root::Root;

pub(super) fn analyze_template(root: &Root, tables: &mut AnalysisTables) {
    let mut context = context::TemplateAnalyzerContext::new(tables, root.span);
    fragment::analyze_fragment(&root.fragment, &mut context);
    diagnostics::emit_assignment_diagnostics(tables);
}
