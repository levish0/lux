use lux_parser::error::ErrorKind;
use lux_parser::parse;
use oxc_allocator::Allocator;

#[test]
fn root_only_svelte_meta_tag_must_be_top_level() {
    let allocator = Allocator::default();
    let result = parse(
        "{#if ok}<svelte:head></svelte:head>{/if}",
        &allocator,
        false,
    );

    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("only valid at the top level"))
    );
}

#[test]
fn duplicate_root_only_svelte_meta_tag_reports_error() {
    let allocator = Allocator::default();
    let result = parse(
        "<svelte:head></svelte:head><svelte:head></svelte:head>",
        &allocator,
        false,
    );

    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("Duplicate root-only Svelte meta tag"))
    );
}

#[test]
fn duplicate_top_level_script_reports_error() {
    let allocator = Allocator::default();
    let result = parse(
        "<script>let a = 1;</script><script>let b = 2;</script>",
        &allocator,
        false,
    );

    assert!(
        result
            .errors
            .iter()
            .any(|e| e.kind == ErrorKind::InvalidScript)
    );
}

#[test]
fn duplicate_top_level_style_reports_error() {
    let allocator = Allocator::default();
    let result = parse(
        "<style>.a { color: red; }</style><style>.b { color: blue; }</style>",
        &allocator,
        false,
    );

    assert!(
        result
            .errors
            .iter()
            .any(|e| e.kind == ErrorKind::InvalidCss)
    );
}

#[test]
fn script_module_attribute_with_value_reports_error() {
    let allocator = Allocator::default();
    let result = parse(
        "<script module=\"yes\">let a = 1;</script>",
        &allocator,
        false,
    );

    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("module` attribute"))
    );
}

#[test]
fn script_context_attribute_must_be_module() {
    let allocator = Allocator::default();
    let result = parse(
        "<script context=\"client\">let a = 1;</script>",
        &allocator,
        false,
    );

    assert!(result.errors.iter().any(|e| {
        e.message
            .contains("context` attribute on <script> must be \"module\"")
    }));
}
