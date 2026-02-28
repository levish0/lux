mod program;

use lux_ast::analysis::{AnalysisTables, ScriptTarget};
use lux_ast::template::root::Root;

pub(super) fn analyze_scripts(root: &Root, tables: &mut AnalysisTables) {
    if let Some(module_script) = &root.module {
        program::analyze_program(&module_script.content, ScriptTarget::Module, tables);
    }

    if let Some(instance_script) = &root.instance {
        program::analyze_program(&instance_script.content, ScriptTarget::Instance, tables);
    }
}
