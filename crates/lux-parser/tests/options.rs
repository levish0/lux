use lux_ast::template::root::{CssOption, CustomElementShadow, Namespace};
use lux_parser::parse;
use oxc_allocator::Allocator;

#[test]
fn parses_basic_svelte_options() {
    let allocator = Allocator::default();
    let result = parse(
        "<svelte:options runes immutable={false} namespace=\"svg\" css=\"injected\" preserveWhitespace accessors={true} customElement=\"x-foo\" />",
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
fn boolean_options_reject_string_literals() {
    let allocator = Allocator::default();
    let result = parse("<svelte:options immutable=\"false\" />", &allocator, false);

    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("immutable must be true or false"))
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

#[test]
fn custom_element_object_literal_is_parsed() {
    let allocator = Allocator::default();
    let result = parse(
        "<svelte:options customElement={{ tag: 'my-widget', shadow: 'none', props: { name: { type: 'String', reflect: false, attribute: 'name' } }, extend: (ce) => ce }} />",
        &allocator,
        true,
    );

    assert!(result.errors.is_empty());
    let options = result.root.options.as_ref().expect("expected options");
    let custom = options
        .custom_element
        .as_ref()
        .expect("expected customElement");
    assert_eq!(custom.tag, Some("my-widget"));
    assert!(matches!(custom.shadow, Some(CustomElementShadow::None)));
    assert!(custom.props.is_some());
    assert!(custom.extend.is_some());
}

#[test]
fn custom_element_null_is_allowed() {
    let allocator = Allocator::default();
    let result = parse("<svelte:options customElement={null} />", &allocator, false);

    assert!(result.errors.is_empty());
    let options = result.root.options.as_ref().expect("expected options");
    assert!(options.custom_element.is_none());
}

#[test]
fn custom_element_string_expression_is_rejected() {
    let allocator = Allocator::default();
    let result = parse(
        "<svelte:options customElement={'my-widget'} />",
        &allocator,
        false,
    );

    assert!(result.errors.iter().any(|e| {
        e.message
            .contains("\"customElement\" must be a string literal tag name")
    }));
}
