use lux_ast::css::stylesheet::StyleSheetChild;
use lux_parser::error::ErrorKind;
use lux_parser::parse;
use oxc_allocator::Allocator;

#[test]
fn style_parses_basic_rule() {
    let allocator = Allocator::default();
    let result = parse(
        "<style>.a { color: red; }</style><div class=\"a\">x</div>",
        &allocator,
        false,
    );

    assert!(result.errors.is_empty());
    let css = result.root.css.as_ref().expect("expected stylesheet");
    assert_eq!(css.children.len(), 1);
    assert!(matches!(css.children[0], StyleSheetChild::Rule(_)));
}

#[test]
fn style_parses_media_atrule() {
    let allocator = Allocator::default();
    let result = parse(
        "<style>@media (max-width: 600px) { .a { color: blue; } }</style>",
        &allocator,
        false,
    );

    assert!(result.errors.is_empty());
    let css = result.root.css.as_ref().expect("expected stylesheet");
    assert_eq!(css.children.len(), 1);
    assert!(matches!(css.children[0], StyleSheetChild::Atrule(_)));
}

#[test]
fn invalid_style_reports_error_instead_of_panicking() {
    let allocator = Allocator::default();
    let result = parse(
        "<style>.a { color: red; .b { color: blue; }</style>",
        &allocator,
        false,
    );

    assert!(
        result
            .errors
            .iter()
            .any(|err| err.kind == ErrorKind::InvalidCss)
    );
    assert!(result.root.css.is_some());
}

#[test]
fn style_parses_nested_pseudo_selector_lists() {
    let allocator = Allocator::default();
    let result = parse(
        "<style>:is(.a, .b):where(.c, .d) { color: red; }</style>",
        &allocator,
        false,
    );

    assert!(result.errors.is_empty());
    let css = result.root.css.as_ref().expect("expected stylesheet");
    assert_eq!(css.children.len(), 1);
    assert!(matches!(css.children[0], StyleSheetChild::Rule(_)));
}

#[test]
fn style_parses_global_with_has_selector() {
    let allocator = Allocator::default();
    let result = parse(
        "<style>:global(.foo):has(.bar) { color: red; }</style>",
        &allocator,
        false,
    );

    assert!(result.errors.is_empty());
    let css = result.root.css.as_ref().expect("expected stylesheet");
    assert_eq!(css.children.len(), 1);
    assert!(matches!(css.children[0], StyleSheetChild::Rule(_)));
}

#[test]
fn style_parses_escaped_class_identifier() {
    let allocator = Allocator::default();
    let result = parse(
        "<style>.foo\\:bar { color: red; }</style>",
        &allocator,
        false,
    );

    assert!(result.errors.is_empty());
    let css = result.root.css.as_ref().expect("expected stylesheet");
    assert_eq!(css.children.len(), 1);
    assert!(matches!(css.children[0], StyleSheetChild::Rule(_)));
}
