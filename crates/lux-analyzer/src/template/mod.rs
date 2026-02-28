mod binding;
mod context;
mod fragment;
mod node;

use lux_ast::analysis::AnalysisTables;
use lux_ast::template::root::Root;

pub(super) fn analyze_template(root: &Root, tables: &mut AnalysisTables) {
    let mut context = context::TemplateAnalyzerContext::new(tables, root.span);
    fragment::analyze_fragment(&root.fragment, &mut context);
}
