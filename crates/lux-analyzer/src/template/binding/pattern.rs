use oxc_ast::ast::BindingPattern;

use super::CollectedBinding;

pub(super) fn collect_pattern_bindings<'a>(
    pattern: &'a BindingPattern<'a>,
) -> Vec<CollectedBinding<'a>> {
    pattern
        .get_binding_identifiers()
        .into_iter()
        .map(|identifier| CollectedBinding {
            name: identifier.name.as_str(),
            span: identifier.span,
        })
        .collect()
}
