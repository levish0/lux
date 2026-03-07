mod program;

use lux_ast::analysis::{AnalysisTables, ScriptTarget};
use lux_ast::template::root::Root;

pub(super) fn analyze_scripts(root: &Root, tables: &mut AnalysisTables) {
    let is_custom_element = root
        .options
        .as_ref()
        .and_then(|options| options.custom_element.as_ref())
        .is_some();
    let component_runes = root
        .options
        .as_ref()
        .and_then(|options| options.runes)
        .unwrap_or_else(|| {
            root.module
                .as_ref()
                .is_some_and(|script| program::has_known_rune(&script.content))
                || root
                    .instance
                    .as_ref()
                    .is_some_and(|script| program::has_known_rune(&script.content))
        });

    if let Some(module_script) = &root.module {
        program::analyze_program(
            &module_script.content,
            ScriptTarget::Module,
            is_custom_element,
            component_runes,
            tables,
        );
    }

    if let Some(instance_script) = &root.instance {
        program::analyze_program(
            &instance_script.content,
            ScriptTarget::Instance,
            is_custom_element,
            component_runes,
            tables,
        );
    }
}
