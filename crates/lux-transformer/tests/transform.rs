use lux_analyzer::analyze;
use lux_parser::parse;
use lux_transformer::transform;
use lux_utils::hash::hash;
use oxc_allocator::Allocator;

#[test]
fn transform_includes_css_hash_and_scope_for_style_blocks() {
    let source = "<style>h1 { color: red; }</style><h1>Hello</h1>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    let expected_css = parsed
        .root
        .css
        .as_ref()
        .map(|stylesheet| stylesheet.content_styles.to_string())
        .expect("expected style block");
    let expected_hash = hash(&expected_css);
    let expected_scope = format!("svelte-{expected_hash}");

    let rendered_css = result.css.as_deref().expect("expected transformed css");
    assert!(rendered_css.contains("color: red;"));
    assert!(rendered_css.contains(&format!("h1.{expected_scope}")));
    assert_eq!(result.css_hash.as_deref(), Some(expected_hash.as_str()));
    assert_eq!(result.css_scope.as_deref(), Some(expected_scope.as_str()));
    assert_eq!(result.js, "export default {};\n");
}

#[test]
fn transform_has_no_css_payload_without_style_blocks() {
    let source = "<h1>Hello</h1>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert_eq!(result.css, None);
    assert_eq!(result.css_hash, None);
    assert_eq!(result.css_scope, None);
    assert_eq!(result.js, "export default {};\n");
}

#[test]
fn transform_removes_global_wrapper_without_scoping() {
    let source = "<style>:global(h1) { color: red; }</style><h1>Hello</h1>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    let css = result.css.expect("expected transformed css");
    assert!(css.contains("h1 { color: red; }"));
    assert!(!css.contains(":global("));
    let scope = result.css_scope.expect("expected css scope");
    assert!(!css.contains(&format!("h1.{scope}")));
}

#[test]
fn transform_scopes_local_part_of_mixed_global_selector() {
    let source = "<style>.a :global(.b) { color: red; }</style><div class=\"a\"><span class=\"b\"></span></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    let scope = result.css_scope.expect("expected css scope");
    let css = result.css.expect("expected transformed css");

    assert!(css.contains(&format!(".a.{scope}")));
    assert!(css.contains(".b"));
    assert!(!css.contains(":global("));
}
