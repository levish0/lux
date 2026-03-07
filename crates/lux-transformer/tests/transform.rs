use lux_analyzer::analyze;
use lux_parser::parse;
use lux_transformer::{TransformTarget, transform, transform_for_target, transform_with_filename};
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
fn transform_uses_filename_for_css_hash_when_available() {
    let source = "<style>h1 { color: red; }</style><h1>Hello</h1>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let filename = "src/routes/+page.svelte";
    let result = transform_with_filename(&parsed.root, &analysis, Some(filename));

    let expected_hash = hash(filename);
    let expected_scope = format!("svelte-{expected_hash}");

    let rendered_css = result.css.as_deref().expect("expected transformed css");
    assert!(rendered_css.contains("color: red;"));
    assert!(rendered_css.contains(&format!("h1.{expected_scope}")));
    assert_eq!(result.css_hash.as_deref(), Some(expected_hash.as_str()));
    assert_eq!(result.css_scope.as_deref(), Some(expected_scope.as_str()));
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
    assert!(result.runtime_modules.is_empty());
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

    assert!(result.js.contains("__lux_stringify("));
    assert!(result.js.contains("_props"));
    assert!(result.js.contains("function({ name })"));
    assert!(result.js.contains("return name;"));
    assert!(result.js.contains("__lux_chunks.push(["));
    assert!(result.js.contains("\"<p\","));
    assert!(result.js.contains("</p>"));
    assert!(!result.js.contains("<!--lux:dynamic:expression-->"));
    assert_eq!(result.runtime_modules.len(), 1);
    assert_eq!(result.runtime_modules[0].specifier, "lux/runtime/server");
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function stringify")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function mount_head")
    );
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
fn transform_typescript_each_block_preserves_context_index_and_key_bindings() {
    let source = "{#each text as line, i (i)}<p>{line}</p>{/each}";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, true);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("return text;"));
    assert!(result.js.contains(".map(function(line, i)"));
    assert!(!result.js.contains("return i(i);"));
    assert!(!result.js.contains("function({ line })"));
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
    assert!(
        result
            .js
            .contains("typeof __lux_await_value.then === \"function\"")
    );
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

    assert!(result.js.contains("from \"lux/runtime/server\";"));
    assert!(result.js.contains("stringify as __lux_stringify"));
    assert!(result.js.contains("escape as __lux_escape"));
    assert!(result.js.contains("escape_attr as __lux_escape_attr"));
    assert!(!result.js.contains("replaceAll("));
    assert!(result.js.contains("__lux_stringify(function({ value })"));
}

#[test]
fn transform_generates_component_runtime_render_path() {
    let source = "<Child msg={name}>hi</Child>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("__lux_render_component("));
    assert!(result.js.contains("const __lux_component_props = {"));
    assert!(result.js.contains("msg: function({ name })"));
    assert!(result.js.contains("children: function()"));
    assert!(!result.js.contains("<!--lux:dynamic:component-->"));
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function render_component")
    );
}

#[test]
fn transform_component_props_include_events_object() {
    let source = "<Child on:click={handle} on:change={onChange} />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("$$events"));
    assert!(result.js.contains("click: function({ handle })"));
    assert!(result.js.contains("change: function({ onChange })"));
}

#[test]
fn transform_component_props_group_duplicate_event_handlers() {
    let source = "<Child on:click={a} on:click={b} />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("$$events"));
    assert!(result.js.contains("click: ["));
    assert!(result.js.contains("function({ a })"));
    assert!(result.js.contains("function({ b })"));
}

#[test]
fn transform_component_props_wrap_once_event_handlers() {
    let source = "<Child on:done|once={handle} />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("$$events"));
    assert!(result.js.contains("done: __lux_once("));
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function once")
    );
}

#[test]
fn transform_component_props_forward_events_without_expression() {
    let source = "<Child on:click />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("$$events"));
    assert!(result.js.contains("_props.$$events"));
    assert!(result.js.contains("click"));
}

#[test]
fn transform_generates_svelte_element_runtime_render_path() {
    let source = "<svelte:element this={tag} foo={x}>ok</svelte:element>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(
        result
            .js
            .contains("const __lux_tag = __lux_stringify(function({ tag })")
    );
    assert!(result.js.contains("return ["));
    assert!(result.js.contains("\"<\","));
    assert!(result.js.contains("\"</\","));
    assert!(result.js.contains("__lux_tag"));
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

    assert!(result.js.contains("from \"lux/runtime/server\";"));
    assert!(result.js.contains("__lux_attributes("));
    assert!(result.js.contains("active:"));
    assert!(result.js.contains("color:"));
    assert!(!result.js.contains("\" class=\\\"\""));
    assert!(!result.js.contains("\" style=\\\"color: \""));
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function class_attr")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function style_attr")
    );
}

#[test]
fn transform_renders_textarea_value_as_body_content_in_ssr() {
    let source = "<script>export let foo = 42;</script><textarea value={foo}/>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("<textarea"));
    assert!(result.js.contains("</textarea>"));
    assert!(!result.js.contains("__lux_attr(\"value\""));
}

#[test]
fn transform_merges_static_and_directive_class_style_into_single_spread_call() {
    let source = "<div {...attrs} class=\"hero\" class:active={ok} style=\"color: red\" style:background={bg}></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("__lux_attributes("));
    assert!(result.js.contains("\"hero\""));
    assert!(result.js.contains("\"color: red\""));
    assert!(result.js.contains("active:"));
    assert!(result.js.contains("background:"));
}

#[test]
fn transform_merges_class_directive_with_static_class_attribute() {
    let source = "<div class=\"hero\" class:active={ok}></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("__lux_class_attr("));
    assert!(result.js.contains("__lux_attr(\"class\""));
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function class_attr")
    );
}

#[test]
fn transform_merges_style_directive_with_static_style_attribute() {
    let source = "<div style=\"color: red\" style:background={bg}></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("__lux_style_attr("));
    assert!(result.js.contains("__lux_attr(\"style\""));
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function style_attr")
    );
}

#[test]
fn transform_re_emits_instance_import_and_handles_dotted_component_name() {
    let source = "<script>import * as Tabs from './tabs';</script><Tabs.Root />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("import * as Tabs from \"./tabs\";"));
    assert!(result.js.contains("const __lux_component = Tabs.Root;"));
    assert!(!result.js.contains("function({ Tabs })"));
}

#[test]
fn transform_import_emit_and_scope_seed_are_driven_by_analysis_table() {
    let source = "<script>import * as Tabs from './tabs';</script><Tabs.Root />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let mut analysis = analyze(&parsed.root);
    analysis.script_imports.clear();
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("import * as Tabs from \"./tabs\";"));
    assert!(result.js.contains("function({ Tabs })"));
}

#[test]
fn transform_emits_instance_imports_before_module_imports() {
    let source = r#"
<script context="module">
  import { m } from './m';
</script>
<script>
  import x from './x';
</script>
<p>{x}</p>
"#;
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    let instance_idx = result
        .js
        .find("import x from \"./x\";")
        .expect("instance import missing");
    let module_idx = result
        .js
        .find("import { m } from \"./m\";")
        .expect("module import missing");
    assert!(instance_idx < module_idx);
}

#[test]
fn transform_strips_typescript_type_only_import_syntax() {
    let source = r#"
<script lang="ts">
  import type { A } from './types';
  import { type B, c } from './mixed';
  import {} from './side-effect';
</script>
{c}
"#;
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("import type"));
    assert!(!result.js.contains("type B"));
    assert!(result.js.contains("import { c } from \"./mixed\";"));
    assert!(result.js.contains("import \"./side-effect\";"));
    assert!(!result.js.contains("from \"./types\""));
    assert_js_parses_as_module(&result.js);
}

#[test]
fn transform_emits_instance_script_declarations_and_uses_local_scope() {
    let source = "<script>let x = 1;</script><p>{x}</p>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("let x = 1;"));
    assert!(!result.js.contains("function({ x })"));
}

#[test]
fn transform_lowers_state_and_derived_runes_in_variable_initializers() {
    let source = "<script>let count = $state(1); let doubled = $derived.by(() => count * 2);</script>{doubled}";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("$state"));
    assert!(!result.js.contains("$derived.by"));
    assert!(result.js.contains("let count = 1;"));
    assert!(result.js.contains("let doubled = (() => count * 2)();"));
}

#[test]
fn transform_drops_effect_rune_expression_statements() {
    let source = "<script>$effect(() => { console.log('x'); });</script><p>ok</p>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("$effect"));
    assert!(!result.js.contains("console.log('x')"));
}

#[test]
fn transform_maps_props_rune_to_runtime_props_object() {
    let source = "<script>let { a } = $props();</script>{a}";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("$props"));
    assert!(result.js.contains("let { a } = _props;"));
    assert!(!result.js.contains("function({ a })"));
}

#[test]
fn transform_maps_props_id_rune_to_runtime_expression() {
    let source = "<script>const id = $props.id();</script><div id={id}></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("$props.id"));
    assert!(result.js.contains("props_id as __lux_props_id"));
    assert!(result.js.contains("const id = __lux_props_id();"));
    assert_js_parses_as_module(&result.js);
}

#[test]
fn transform_rewrites_bindable_default_in_props_destructure() {
    let source = "<script>let { value = $bindable() } = $props();</script>{value}";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("$bindable"));
    assert!(result.js.contains("let { value = undefined } = _props;"));
    assert!(!result.js.contains("function({ value })"));
    assert_js_parses_as_module(&result.js);
}

#[test]
fn transform_emits_module_script_statements() {
    let source =
        "<script context=\"module\">const n = 1; function f() { return n; }</script><p>x</p>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("const n = 1;"));
    assert!(result.js.contains("function f()"));
}

#[test]
fn transform_resolves_module_script_bindings_in_template_scope() {
    let source = r#"
<script context="module">
  export const buttonVariants = (options) => options.variant;
</script>
<p>{buttonVariants({ variant: 'ok' })}</p>
"#;
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("export const buttonVariants"));
    assert!(result.js.contains("options.variant"));
    assert!(result.js.contains("buttonVariants({"));
    assert!(!result.js.contains("function({ buttonVariants })"));
    assert!(!result.js.contains("_props.buttonVariants"));
}

#[test]
fn transform_legacy_export_let_reads_from_props() {
    let source = "<script>export let src; export let solid = false;</script><p>{src}{solid}</p>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("let src = _props.src;"));
    assert!(
        result
            .js
            .contains("_props.solid === undefined ? false : _props.solid")
    );
}

#[test]
fn transform_reactive_assignment_declares_local_binding() {
    let source = "<script>export let src; export let solid = false; $: icon = src?.[solid ? 'solid' : 'default'];</script><svg {...icon?.a}></svg>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("let icon;"));
    assert!(
        result
            .js
            .contains("icon = src?.[solid ? \"solid\" : \"default\"];")
    );
    assert!(!result.js.contains("function({ icon })"));
    assert!(!result.js.contains("_props.icon"));
}

#[test]
fn transform_legacy_rest_props_uses_runtime_helper() {
    let source =
        "<script>export let src; export let solid = false;</script><svg {...$$restProps}></svg>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("const $$props = _props;"));
    assert!(
        result
            .js
            .contains("const $$restProps = __lux_rest_props(_props, [\"solid\", \"src\"]);")
            || result
                .js
                .contains("const $$restProps = __lux_rest_props(_props, [\"src\", \"solid\"]);")
    );
    assert!(!result.js.contains("_props.$$restProps"));
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function rest_props")
    );
}

#[test]
fn transform_ts_instance_script_output_is_valid_javascript() {
    let source = "<script lang=\"ts\">let x: number = 1;</script>{x}";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);
    assert_js_parses_as_module(&result.js);
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
    assert!(result.js.contains("const __lux_template = \"\";"));
    assert!(!result.js.contains("__lux_begin_render"));
    assert!(
        result
            .js
            .contains("const __lux_head_html = \"<title>t</title>\";")
    );
}

#[test]
fn transform_excludes_svelte_head_from_body_template_when_dynamic() {
    let source = "<script>let title = 't';</script><svelte:head><title>{title}</title></svelte:head><p>{title}</p>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("head: function"));
    assert!(result.js.contains("const __lux_template = \"<p></p>\";"));
    assert!(!result.js.contains("const __lux_template = \"<title>"));
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
    assert!(result.js.contains("__lux_stringify(x)"));
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
    let source = "<Child>default<p slot=\"title\">t</p><svelte:fragment slot=\"footer\"><i>f</i></svelte:fragment></Child>";
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
    assert!(result.js.contains("__lux_attr(\"slot\""));
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
    assert!(result.js.contains("__lux_stringify(item)"));
    assert!(!result.js.contains("function({ item })"));
    assert!(!result.js.contains("children: function("));
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
    assert!(result.js.contains("__lux_stringify(value)"));
}

#[test]
fn transform_adds_load_error_capture_for_on_directives() {
    let source = "<img on:load={ready} on:error={failed}>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains(" onload=\\\"this.__e=event\\\""));
    assert!(result.js.contains(" onerror=\\\"this.__e=event\\\""));
}

#[test]
fn transform_adds_load_error_capture_for_spread_and_use() {
    let source = "<img {...attrs} use:enhance>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains(" onload=\\\"this.__e=event\\\""));
    assert!(result.js.contains(" onerror=\\\"this.__e=event\\\""));
}

#[test]
fn transform_does_not_add_load_error_capture_for_non_load_error_elements() {
    let source = "<div on:load={ready} {...attrs} use:enhance></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains(" onload=\\\"this.__e=event\\\""));
    assert!(!result.js.contains(" onerror=\\\"this.__e=event\\\""));
}

#[test]
fn transform_plain_onload_attribute_is_captured_for_load_error_elements() {
    let source = "<img onload=\"alert(1)\">";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains(" onload=\\\"this.__e=event\\\""));
    assert!(!result.js.contains("alert(1)"));
}

#[test]
fn transform_plain_event_attributes_are_omitted_on_non_load_error_elements() {
    let source = "<div onclick=\"alert(1)\">x</div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("onclick"));
    assert!(!result.js.contains("alert(1)"));
}

#[test]
fn transform_omits_bind_this_attribute_on_regular_element() {
    let source = "<div bind:this={el}></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains(" this=\\\""));
    assert!(!result.js.contains("bind:this"));
}

#[test]
fn transform_omits_bind_this_from_component_props() {
    let source = "<Child bind:this={child} foo={x} />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("foo: function({ x })"));
    assert!(!result.js.contains("this: function({ child })"));
}

#[test]
fn transform_component_bind_emits_getter_and_setter_props() {
    let source = "<Child bind:value={value} />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("get value()"));
    assert!(result.js.contains("set value($$value)"));
    assert!(result.js.contains("_props.value = $$value"));
}

#[test]
fn transform_component_bind_uses_local_setter_target_when_available() {
    let source = "<script>let value = '';</script><Child bind:value={value} />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("set value($$value)"));
    assert!(result.js.contains("value = $$value"));
    assert!(!result.js.contains("_props.value = $$value"));
}

#[test]
fn transform_component_bind_sequence_expression_emits_accessor_calls() {
    let source = "<script>let value = ''; const get = () => value; const set = (v) => value = v;</script><Child bind:value={get, set} />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("get value()"));
    assert!(result.js.contains("return get()"));
    assert!(result.js.contains("set value($$value)"));
    assert!(result.js.contains("set($$value)"));
}

#[test]
fn transform_component_bind_this_emits_assignment_side_effect() {
    let source = "<script>let child;</script><Child bind:this={child} />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("this:"));
    assert!(result.js.contains("child = __lux_component"));
}

#[test]
fn transform_component_bind_this_falls_back_to_props_assignment_when_not_local() {
    let source = "<Child bind:this={child} />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("_props.child = __lux_component"));
}

#[test]
fn transform_component_bind_this_sequence_expression_calls_setter() {
    let source = "<script>let child; const get = () => child; const set = (v) => child = v;</script><Child bind:this={get, set} />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(!result.js.contains("this:"));
    assert!(result.js.contains("set(__lux_component)"));
}

#[test]
fn transform_svelte_component_bind_this_emits_assignment_side_effect() {
    let source =
        "<script>let child; let Comp;</script><svelte:component this={Comp} bind:this={child} />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform(&parsed.root, &analysis);

    assert!(result.js.contains("const __lux_component = Comp;"));
    assert!(result.js.contains("child = __lux_component"));
}

#[test]
fn transform_client_target_emits_mountable_default_export() {
    let source = "<p>{name}</p>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("from \"lux/runtime/client\";"));
    assert!(
        result.js.contains("export default"),
        "missing default function export in client js:\n{}",
        result.js
    );
    assert!(result.js.contains("__lux_is_mount_target($$anchor)"));
    assert!(result.js.contains("__lux_cleanup_mount($$anchor)"));
    assert!(result.js.contains("__lux_mount_html($$anchor, __lux_html)"));
    assert!(
        result
            .js
            .contains("__lux_mount_actions($$anchor, __lux_actions)")
    );
    assert!(
        result
            .js
            .contains("__lux_mount_transitions($$anchor, __lux_transitions)")
    );
    assert!(
        result
            .js
            .contains("__lux_mount_animations($$anchor, __lux_animations)")
    );
    assert_eq!(result.runtime_modules.len(), 1);
    assert_eq!(result.runtime_modules[0].specifier, "lux/runtime/client");
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function mount_html")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function mount_head")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function cleanup_mount")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function transition_attr")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function mount_transitions")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function animate_attr")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function mount_animations")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("const anchor_regions = new WeakMap();")
    );
    assert_js_parses_as_module(&result.js);
}

#[test]
fn transform_client_target_mounts_svelte_head() {
    let source = "<script>let title = 't';</script><svelte:head><title>{title}</title></svelte:head><p>{title}</p>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("const __lux_head_html = "));
    assert!(
        result
            .js
            .contains("__lux_mount_head($$anchor, __lux_head_html)")
    );
    assert!(result.js.contains("const __lux_html = "));
    assert!(result.js.contains("__lux_mount_html($$anchor, __lux_html)"));
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function mount_head")
    );
}

#[test]
fn transform_client_target_includes_runtime_module_for_static_templates() {
    let source = "<h1>Hello</h1>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("from \"lux/runtime/client\";"));
    assert_eq!(result.runtime_modules.len(), 1);
    assert_eq!(result.runtime_modules[0].specifier, "lux/runtime/client");
}

#[test]
fn transform_client_target_emits_event_runtime_hooks_for_on_directive() {
    let source = "<button on:click={handle}>Tap</button>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_event_attr(\"click\""));
    assert!(
        result
            .js
            .contains("__lux_mount_events($$anchor, __lux_events)")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function mount_events")
    );
}

#[test]
fn transform_client_target_preserves_on_directive_modifiers() {
    let source = "<button on:click|once|capture|preventDefault={handle}>Tap</button>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_event_attr(\"click\""));
    assert!(result.js.contains("\"once\""));
    assert!(result.js.contains("\"capture\""));
    assert!(result.js.contains("\"preventDefault\""));
}

#[test]
fn transform_client_target_emits_bind_runtime_hooks_for_value() {
    let source = "<input bind:value={value}>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_bind_attr(\"value\""));
    assert!(
        result
            .js
            .contains("__lux_mount_bindings($$anchor, __lux_bindings)")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function mount_bindings")
    );
}

#[test]
fn transform_client_target_emits_bind_runtime_hooks_for_this() {
    let source = "<input bind:this={el}>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_bind_attr(\"this\""));
    assert!(
        result
            .js
            .contains("__lux_mount_bindings($$anchor, __lux_bindings)")
    );
}

#[test]
fn transform_client_target_emits_bind_runtime_hooks_for_checked() {
    let source = "<input type=\"checkbox\" bind:checked={done}>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_bind_attr(\"checked\""));
}

#[test]
fn transform_client_target_emits_global_window_event_and_bind_hooks() {
    let source = "<svelte:window on:resize={onResize} bind:innerWidth={width} />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(
        result
            .js
            .contains("__lux_event_target_attr(\"window\", \"resize\""),
        "{}",
        result.js
    );
    assert!(
        result
            .js
            .contains("__lux_bind_target_attr(\"window\", \"innerWidth\""),
        "{}",
        result.js
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function event_target_attr")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function bind_target_attr")
    );
}

#[test]
fn transform_client_target_emits_global_document_event_and_bind_hooks() {
    let source =
        "<svelte:document on:visibilitychange={onVisibility} bind:visibilityState={state} />";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(
        result
            .js
            .contains("__lux_event_target_attr(\"document\", \"visibilitychange\""),
        "{}",
        result.js
    );
    assert!(
        result
            .js
            .contains("__lux_bind_target_attr(\"document\", \"visibilityState\""),
        "{}",
        result.js
    );
}

#[test]
fn transform_client_target_emits_bind_runtime_hooks_for_group() {
    let source = "<input type=\"checkbox\" bind:group={selected} value=\"a\">";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_bind_attr(\"group\""));
    assert!(
        result.runtime_modules[0]
            .code
            .contains("function apply_group_binding")
    );
}

#[test]
fn transform_server_target_renders_checked_attribute_for_group_bindings() {
    let source = "<script>let selected = ['a']; let choice = 'x';</script><input type=\"checkbox\" bind:group={selected} value=\"a\"><input type=\"radio\" bind:group={choice} value=\"x\">";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Server);

    assert!(
        result.js.contains("__lux_attr(\"checked\"") && result.js.contains("selected.includes"),
        "{}",
        result.js
    );
    assert!(
        result.js.contains("__lux_attr(\"checked\"") && result.js.contains("choice ==="),
        "{}",
        result.js
    );
}

#[test]
fn transform_server_target_omits_value_attribute_for_file_input_binding() {
    let source = "<script>let value = '/tmp/file';</script><input type=\"file\" bind:value={value}>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Server);

    assert!(!result.js.contains("__lux_attr(\"value\""), "{}", result.js);
    assert!(result.js.contains("__lux_bind_attr(\"value\""), "{}", result.js);
}

#[test]
fn transform_server_target_imports_rest_props_helper_for_exported_props() {
    let source = "<script>export let x = 1;</script><div>{x}</div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Server);

    assert!(
        result
            .js
            .contains("rest_props as __lux_rest_props"),
        "{}",
        result.js
    );
    assert!(result.js.contains("const $$restProps = __lux_rest_props"), "{}", result.js);
}

#[test]
fn transform_client_target_emits_bind_runtime_hooks_for_files() {
    let source = "<input type=\"file\" bind:files={files}>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_bind_attr(\"files\""));
    assert!(
        result.runtime_modules[0]
            .code
            .contains("function set_element_files")
    );
}

#[test]
fn transform_client_target_bind_value_on_select_uses_runtime_select_path() {
    let source = "<select bind:value={choice}><option value=\"a\">a</option></select>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_bind_attr(\"value\""));
    assert!(!result.js.contains("<select value=\\\""));
    assert!(
        result.runtime_modules[0]
            .code
            .contains("function set_select_value")
    );
}

#[test]
fn transform_client_target_bind_setter_assigns_local_identifier() {
    let source = "<script>let value = '';</script><input bind:value={value}>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("value = __lux_value"));
    assert!(!result.js.contains("_props.value = __lux_value"));
}

#[test]
fn transform_client_target_bind_setter_assigns_local_member_expression() {
    let source = "<script>let state = { value: '' };</script><input bind:value={state.value}>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("state.value = __lux_value"));
}

#[test]
fn transform_client_target_bind_setter_assigns_local_computed_member_expression() {
    let source = "<script>let state = { value: '' }; let key = 'value';</script><input bind:value={state[key]}>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("state[key] = __lux_value"));
}

#[test]
fn transform_client_target_bind_sequence_expression_uses_explicit_getter_setter() {
    let source = "<script>let value = ''; const get = () => value; const set = (v) => value = v;</script><input bind:value={get, set}>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_bind_attr(\"value\", get, set)"));
}

#[test]
fn transform_client_target_emits_bind_runtime_hooks_for_open() {
    let source = "<details bind:open={isOpen}></details>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_bind_attr(\"open\""));
    assert!(result.js.contains("__lux_attr(\"open\""));
}

#[test]
fn transform_client_target_emits_bind_runtime_hooks_for_indeterminate() {
    let source = "<input type=\"checkbox\" bind:indeterminate={partial}>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_bind_attr(\"indeterminate\""));
    assert!(
        result.runtime_modules[0]
            .code
            .contains("function set_element_indeterminate")
    );
}

#[test]
fn transform_client_target_emits_use_runtime_hooks() {
    let source = "<script>const action = () => {};</script><div use:action></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_use_attr(\"action\""));
    assert!(
        result
            .js
            .contains("__lux_mount_actions($$anchor, __lux_actions)")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function use_attr")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function mount_actions")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("function apply_action")
    );
}

#[test]
fn transform_client_target_supports_dotted_use_directive_name() {
    let source = "<script>const ns = { action: () => {} };</script><div use:ns.action></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_use_attr(\"ns.action\""));
    assert!(result.js.contains("ns.action"));
}

#[test]
fn transform_client_target_emits_attach_runtime_hooks() {
    let source = "<script>const attach = () => {};</script><div {@attach attach}></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_use_attr(\"attach\""));
    assert!(
        result
            .js
            .contains("__lux_mount_actions($$anchor, __lux_actions)")
    );
}

#[test]
fn transform_client_target_emits_transition_runtime_hooks() {
    let source = "<script>const fade = () => ({ in() {}, out() {}, destroy() {} });</script><div transition:fade></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_transition_attr(\"fade\""));
    assert!(
        result
            .js
            .contains("__lux_mount_transitions($$anchor, __lux_transitions)")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function transition_attr")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function mount_transitions")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("function apply_transition")
    );
}

#[test]
fn transform_client_target_emits_animate_runtime_hooks() {
    let source = "<script>const flip = () => ({ destroy() {} });</script><div animate:flip></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_animate_attr(\"flip\""));
    assert!(
        result
            .js
            .contains("__lux_mount_animations($$anchor, __lux_animations)")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function animate_attr")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("export function mount_animations")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("function apply_animation")
    );
}

#[test]
fn transform_client_target_emits_bind_runtime_hooks_for_contenteditable_properties() {
    let source = "<div contenteditable bind:textContent={value}></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_bind_attr(\"textContent\""));
    assert!(
        result.runtime_modules[0]
            .code
            .contains("function set_element_text_property")
    );
}

#[test]
fn transform_client_target_runtime_includes_media_bind_handlers() {
    let source = "<video bind:currentTime={t} bind:paused={p} bind:duration={d}></video>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_bind_attr(\"currentTime\""));
    assert!(result.js.contains("__lux_bind_attr(\"paused\""));
    assert!(result.js.contains("__lux_bind_attr(\"duration\""));
    assert!(
        result.runtime_modules[0]
            .code
            .contains("function apply_media_binding")
    );
}

#[test]
fn transform_client_target_runtime_includes_size_bind_handlers() {
    let source = "<div bind:clientWidth={w} bind:contentRect={rect}></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);

    assert!(result.js.contains("__lux_bind_attr(\"clientWidth\""));
    assert!(result.js.contains("__lux_bind_attr(\"contentRect\""));
    assert!(
        result.runtime_modules[0]
            .code
            .contains("function apply_size_binding")
    );
    assert!(
        result.runtime_modules[0]
            .code
            .contains("function apply_resize_observer_binding")
    );
}

#[test]
fn transform_client_target_runtime_tracks_mount_cleanup_state() {
    let source = "<div on:click={handle} bind:this={el}></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);
    let runtime = &result.runtime_modules[0].code;

    assert!(runtime.contains("const anchor_mount_state = new WeakMap();"));
    assert!(runtime.contains("clear_anchor_mount_state(anchor);"));
    assert!(runtime.contains("function run_cleanup_list"));
}

#[test]
fn transform_client_target_runtime_cleans_bind_this_with_null_on_teardown() {
    let source = "<div bind:this={el}></div>";
    let allocator = Allocator::default();
    let parsed = parse(source, &allocator, false);
    assert!(parsed.errors.is_empty(), "parse should succeed");

    let analysis = analyze(&parsed.root);
    let result = transform_for_target(&parsed.root, &analysis, TransformTarget::Client);
    let runtime = &result.runtime_modules[0].code;

    assert!(runtime.contains("return () => setter(null);"));
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
        js.contains("export default function __lux_component"),
        "missing default export: {js}"
    );
    assert!(
        js.contains("Object.assign(__lux_component, {"),
        "missing metadata assignment: {js}"
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
