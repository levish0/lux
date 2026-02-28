use lux_ast::template::root::{CssOption, Namespace};
use lux_parser::parse;
use oxc_allocator::Allocator;

#[test]
fn parses_basic_svelte_options() {
    let allocator = Allocator::default();
    let result = parse(
        "<svelte:options runes immutable=\"false\" namespace=\"svg\" css=\"injected\" preserveWhitespace accessors=\"true\" customElement=\"x-foo\" />",
        &allocator,
        false,
    );

    assert!(result.errors.is_empty());
    let options = result.root.options.as_ref().expect("expected options");
    assert_eq!(options.runes, Some(true));
    assert_eq!(options.immutable, Some(false));
    assert_eq!(options.namespace, Some(Namespace::Svg));
    assert_eq!(options.css, Some(CssOption::Injected));
    assert_eq!(options.preserve_whitespace, Some(true));
    assert_eq!(options.accessors, Some(true));
    assert_eq!(
        options.custom_element.as_ref().and_then(|ce| ce.tag),
        Some("x-foo")
    );
}

#[test]
fn unknown_svelte_options_attribute_reports_error() {
    let allocator = Allocator::default();
    let result = parse("<svelte:options nope />", &allocator, false);

    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("Unknown svelte:options attribute"))
    );
}

#[test]
fn svelte_options_disallows_children() {
    let allocator = Allocator::default();
    let result = parse(
        "<svelte:options><div /></svelte:options>",
        &allocator,
        false,
    );

    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("svelte:options cannot have child nodes"))
    );
}

#[test]
fn svelte_options_accepts_expression_literals() {
    let allocator = Allocator::default();
    let result = parse(
        "<svelte:options runes={true} immutable={false} css={'injected'} />",
        &allocator,
        false,
    );

    assert!(result.errors.is_empty());
    let options = result.root.options.as_ref().expect("expected options");
    assert_eq!(options.runes, Some(true));
    assert_eq!(options.immutable, Some(false));
    assert_eq!(options.css, Some(CssOption::Injected));
}
