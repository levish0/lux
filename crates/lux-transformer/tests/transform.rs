use lux_analyzer::analyze;
use lux_parser::parse;
use lux_transformer::transform;
use lux_utils::hash::hash;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

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
    assert_component_js_payload(&result.js);
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
    assert_component_js_payload(&result.js);
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

#[test]
fn transform_generates_expression_runtime_render() {
    let source = "<p>{name}</p>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("String("));
    assert!(result.js.contains("_props"));
    assert!(result.js.contains("function({ name })"));
    assert!(result.js.contains("return name;"));
    assert!(result.js.contains("\"<p\" + \">\""));
    assert!(result.js.contains("</p>"));
    assert!(!result.js.contains("<!--lux:dynamic:expression-->"));
}

#[test]
fn transform_generates_if_runtime_render() {
    let source = "{#if ok}<p>A</p>{:else}<p>B</p>{/if}";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("function({ ok })"));
    assert!(result.js.contains("\"A\""));
    assert!(result.js.contains("\"B\""));
}

#[test]
fn transform_generates_each_runtime_render() {
    let source = "{#each items as item}<p>{item}</p>{/each}";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("Array.from("));
    assert!(result.js.contains("_props"));
    assert!(result.js.contains(".map(function(item)"));
    assert!(result.js.contains(".join(\"\")"));
}

#[test]
fn transform_generates_await_runtime_render() {
    let source = "{#await promise then value}<p>{value}</p>{:catch err}<p>{err}</p>{/await}";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("__lux_await_value"));
    assert!(result.js.contains("typeof __lux_await_value.then === \"function\""));
    assert!(result.js.contains("catch (err)"));
}

#[test]
fn transform_generates_snippet_assignment_and_render_call() {
    let source = "{#snippet greet(name)}<p>{name}</p>{/snippet}{@render greet('x')}";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("_props.greet = function(name)"));
    assert!(result.js.contains("greet(\"x\")"));
}

#[test]
fn transform_escapes_expression_tag_but_not_html_tag() {
    let source = "<p>{value}</p><p>{@html value}</p>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("replaceAll(\"<\", \"&lt;\")"));
    assert!(result.js.contains("replaceAll(\">\", \"&gt;\")"));
    assert!(result.js.contains("String(function({ value })"));
}

#[test]
fn transform_generates_component_runtime_render_path() {
    let source = "<Child msg={name}>hi</Child>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("typeof __lux_component.render === \"function\""));
    assert!(result.js.contains("const __lux_component_props = {"));
    assert!(result.js.contains("msg: function({ name })"));
    assert!(result.js.contains("children: function()"));
    assert!(!result.js.contains("<!--lux:dynamic:component-->"));
}

#[test]
fn transform_generates_svelte_element_runtime_render_path() {
    let source = "<svelte:element this={tag} foo={x}>ok</svelte:element>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("const __lux_tag = String(function({ tag })"));
    assert!(result.js.contains("\"<\" + __lux_tag"));
    assert!(result.js.contains("\"</\" + __lux_tag + \">\""));
    assert!(!result.js.contains("<!--lux:dynamic:svelte-element-->"));
}

#[test]
fn transform_generates_spread_and_directive_runtime_attributes() {
    let source = "<div {...attrs} class:active={ok} style:color={color}></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("Object.entries("));
    assert!(result.js.contains("__lux_entry[1] === true"));
    assert!(result.js.contains(" ? \" class=\\\"\" + \"active\" + \"\\\"\" : \"\""));
    assert!(result.js.contains("\" style=\\\"color: \" + String("));
}

#[test]
fn transform_keeps_svelte_head_static_when_children_are_static() {
    let source = "<svelte:head><title>t</title></svelte:head>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("const __lux_has_dynamic = false;"));
    assert!(!result.js.contains("<!--lux:dynamic:svelte-head-->"));
    assert!(result.js.contains("<title>t</title>"));
}

#[test]
fn transform_generates_const_tag_runtime_binding() {
    let source = "{@const x = value}<p>{x}</p>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("<!--lux:dynamic:const-tag-->"));
    assert!(result.js.contains("const x = function({ value })"));
    assert!(result.js.contains("String(x ?? \"\")"));
}

#[test]
fn transform_generates_debug_tag_runtime_side_effect() {
    let source = "{@debug name}<p>x</p>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("<!--lux:dynamic:debug-tag-->"));
    assert!(result.js.contains("console.log({"));
    assert!(result.js.contains("name: function({ name })"));
}

#[test]
fn transform_generates_svelte_self_runtime_render_path() {
    let source = "<svelte:self msg={name} />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("<!--lux:dynamic:svelte-self-->"));
    assert!(result.js.contains("_props.__lux_self"));
    assert!(result.js.contains("msg: function({ name })"));
    assert!(result.js.contains("__lux_render"));
}

#[test]
fn transform_generates_slot_element_runtime_fallback_path() {
    let source = "<slot><p>fallback</p></slot>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("<slot"));
    assert!(result.js.contains("__lux_slot_fn"));
    assert!(result.js.contains("<p"));
}

#[test]
fn transform_component_props_include_default_slots_object() {
    let source = "<Child>hi</Child>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("children: function()"));
    assert!(result.js.contains("$$slots"));
    assert!(result.js.contains("default: function()"));
}

#[test]
fn transform_component_groups_named_slots() {
    let source =
        "<Child>default<p slot=\"title\">t</p><svelte:fragment slot=\"footer\"><i>f</i></svelte:fragment></Child>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("children: function()"));
    assert!(result.js.contains("$$slots"));
    assert!(result.js.contains("default: function()"));
    assert!(result.js.contains("title: function()"));
    assert!(result.js.contains("footer: function()"));
    assert!(result.js.contains(" slot=\\\""));
}

#[test]
fn transform_component_children_prop_keeps_default_slot_payload() {
    let source = "<Child children={x}>default</Child>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("children: function({ x })"));
    assert!(!result.js.contains("children: function()"));
    assert!(result.js.contains("$$slots"));
    assert!(result.js.contains("default: function()"));
}

#[test]
fn transform_component_named_slot_only_without_children_prop() {
    let source = "<Child><p slot=\"named\">x</p></Child>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("children: function()"));
    assert!(result.js.contains("$$slots"));
    assert!(result.js.contains("named: function()"));
}

#[test]
fn transform_component_default_slot_let_directive_uses_slot_props() {
    let source = "<Child let:item>{item}</Child>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("default: function(__lux_slot_props)"));
    assert!(result.js.contains("__lux_slot_props.item"));
    assert!(result.js.contains("String(item ?? \"\")"));
    assert!(!result.js.contains("function({ item })"));
}

#[test]
fn transform_component_named_slot_let_directive_uses_slot_props() {
    let source = "<Child><p slot=\"named\" let:value>{value}</p></Child>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("named: function(__lux_slot_props)"));
    assert!(result.js.contains("__lux_slot_props.value"));
    assert!(result.js.contains("String(value ?? \"\")"));
}

fn assert_component_js_payload(js: &str) {
    assert!(
        js.contains("const __lux_template = "),
        "missing template const: {js}"
    );
    assert!(js.contains("const __lux_css = "), "missing css const: {js}");
    assert!(
        js.contains("const __lux_css_hash = "),
        "missing css hash const: {js}"
    );
    assert!(
        js.contains("const __lux_css_scope = "),
        "missing css scope const: {js}"
    );
    assert!(
        js.contains("const __lux_has_dynamic = "),
        "missing has_dynamic const: {js}"
    );
    assert!(
        js.contains("export { __lux_template as template"),
        "missing named export: {js}"
    );
    assert!(
        js.contains("export default {"),
        "missing default export: {js}"
    );
    assert!(
        js.contains("render: function") && js.contains("_props = {}"),
        "missing render parameter default: {js}"
    );
    assert_js_parses_as_module(js);
}

fn assert_js_parses_as_module(js: &str) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, js, SourceType::mjs()).parse();
    assert!(
        parsed.errors.is_empty(),
        "generated js failed to parse: {:?}\nsource:\n{}",
        parsed.errors,
        js
    );
}
