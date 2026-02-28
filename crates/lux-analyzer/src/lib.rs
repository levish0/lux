mod css;
mod script;
mod template;

use lux_ast::analysis::AnalysisTables;
use lux_ast::template::root::Root;

pub fn analyze(root: &Root) -> AnalysisTables {
    let mut tables = AnalysisTables::default();

    script::analyze_scripts(root, &mut tables);
    template::analyze_template(root, &mut tables);

    if let Some(stylesheet) = &root.css {
        css::analyze_stylesheet(stylesheet, &mut tables);
    }

    tables
}
