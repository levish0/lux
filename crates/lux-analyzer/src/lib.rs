mod css;

use lux_ast::analysis::AnalysisTables;
use lux_ast::template::root::Root;

pub fn analyze(root: &Root) -> AnalysisTables {
    let mut tables = AnalysisTables::default();

    if let Some(stylesheet) = &root.css {
        css::analyze_stylesheet(stylesheet, &mut tables);
    }

    tables
}
