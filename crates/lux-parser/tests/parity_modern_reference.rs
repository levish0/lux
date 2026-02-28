use std::fs;
use std::path::PathBuf;

use lux_ast::template::attribute::AttributeNode;
use lux_ast::template::root::FragmentNode;
use lux_parser::parse;
use oxc_allocator::Allocator;
use oxc_span::GetSpan;
use serde_json::{json, Value};

#[test]
#[ignore = "parity tracking against reference parser-modern samples (strict only)"]
fn parity_against_reference_parser_modern_strict() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let samples_dir = manifest_dir.join("tests/fixtures/parser-modern/samples");
    assert!(samples_dir.exists(), "missing {}", samples_dir.display());

    let mut sample_dirs: Vec<PathBuf> = fs::read_dir(&samples_dir)
        .expect("failed to read samples")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    sample_dirs.sort();

    let mut mismatches = Vec::new();

    for sample_dir in sample_dirs {
        let sample_name = sample_dir
            .file_name()
            .and_then(|name| name.to_str())
            .expect("utf-8 directory name");

        // strict-only target
        if sample_name.starts_with("loose-") {
            continue;
        }

        let input_path = sample_dir.join("input.svelte");
        let output_path = sample_dir.join("output.json");
        if !input_path.exists() || !output_path.exists() {
            continue;
        }

        let input_raw = fs::read_to_string(&input_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", input_path.display()));
        let input = normalize_reference_input(&input_raw);

        let expected_raw = fs::read_to_string(&output_path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", output_path.display()));
        let expected_json: Value = serde_json::from_str(&expected_raw)
            .unwrap_or_else(|err| panic!("invalid JSON {}: {err}", output_path.display()));

        let allocator = Allocator::default();
        let actual = parse(&input, &allocator, false);

        let mut expected_lines = Vec::new();
        emit_reference_root(&mut expected_lines, &expected_json, &input, "root");

        let mut actual_lines = Vec::new();
        emit_lux_root(&mut actual_lines, &actual, &input, "root");

        let out_dir = sample_dir.join("_lux_parity");
        if expected_lines != actual_lines {
            let _ = fs::remove_dir_all(&out_dir);
            let _ = fs::create_dir_all(&out_dir);
            let expected_json = json!({ "lines": expected_lines });
            let actual_json = json!({ "lines": actual_lines });
            let _ = fs::write(
                out_dir.join("expected.normalized.json"),
                serde_json::to_string_pretty(&expected_json).unwrap_or_default(),
            );
            let _ = fs::write(
                out_dir.join("actual.normalized.json"),
                serde_json::to_string_pretty(&actual_json).unwrap_or_default(),
            );
            mismatches.push(sample_name.to_string());
        } else if out_dir.exists() {
            let _ = fs::remove_dir_all(out_dir);
        }
    }

    assert!(
        mismatches.is_empty(),
        "mismatches ({}): {}",
        mismatches.len(),
        mismatches.join(", ")
    );
}

fn normalize_reference_input(input: &str) -> String {
    input.replace('\r', "").trim_end().to_string()
}

fn emit_reference_root(lines: &mut Vec<String>, root: &Value, source: &str, path: &str) {
    lines.push(format!("{path}:errors=0"));
    lines.push(format!("{path}:warnings=0"));
    lines.push(format!(
        "{path}:options_present={}",
        !root["options"].is_null()
    ));
    lines.push(format!(
        "{path}:instance_present={}",
        !root["instance"].is_null()
    ));
    lines.push(format!(
        "{path}:module_present={}",
        !root["module"].is_null()
    ));
    lines.push(format!("{path}:css_present={}", !root["css"].is_null()));
    lines.push(format!(
        "{path}:fragment_len={}",
        root["fragment"]["nodes"]
            .as_array()
            .map(|nodes| nodes.len())
            .unwrap_or(0)
    ));

    emit_reference_options(lines, &root["options"], &format!("{path}/options"));
    emit_reference_script(lines, &root["instance"], &format!("{path}/instance"));
    emit_reference_script(lines, &root["module"], &format!("{path}/module"));
    emit_reference_css(lines, &root["css"], &format!("{path}/css"));
    emit_reference_fragment(
        lines,
        &root["fragment"],
        source,
        &format!("{path}/fragment"),
    );
}

fn emit_reference_options(lines: &mut Vec<String>, options: &Value, path: &str) {
    if options.is_null() {
        lines.push(format!("{path}:none"));
        return;
    }
    lines.push(format!("{path}:runes={}", opt_bool_str(&options["runes"])));
    lines.push(format!(
        "{path}:namespace={}",
        opt_str(&options["namespace"])
    ));
    lines.push(format!("{path}:css={}", opt_str(&options["css"])));
    lines.push(format!(
        "{path}:custom_tag={}",
        options["customElement"]["tag"]
            .as_str()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "none".to_string())
    ));
}

fn emit_reference_script(lines: &mut Vec<String>, script: &Value, path: &str) {
    if script.is_null() {
        lines.push(format!("{path}:none"));
        return;
    }
    lines.push(format!(
        "{path}:context={}",
        script["context"].as_str().unwrap_or("")
    ));
    let attrs = script["attributes"]
        .as_array()
        .map(|array| {
            array
                .iter()
                .map(|attr| attr["name"].as_str().unwrap_or("").to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    lines.push(format!("{path}:attrs={attrs:?}"));
}

fn emit_reference_css(lines: &mut Vec<String>, css: &Value, path: &str) {
    if css.is_null() {
        lines.push(format!("{path}:none"));
        return;
    }
    let children = css["children"]
        .as_array()
        .map(|array| {
            array
                .iter()
                .map(|child| child["type"].as_str().unwrap_or("").to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    lines.push(format!("{path}:children={children:?}"));
}

fn emit_reference_fragment(lines: &mut Vec<String>, fragment: &Value, source: &str, path: &str) {
    let nodes = fragment["nodes"].as_array().cloned().unwrap_or_default();
    lines.push(format!("{path}:len={}", nodes.len()));
    for (index, node) in nodes.iter().enumerate() {
        emit_reference_node(lines, node, source, &format!("{path}/{index}"));
    }
}

fn emit_reference_node(lines: &mut Vec<String>, node: &Value, source: &str, path: &str) {
    let kind = node["type"].as_str().unwrap_or("");
    lines.push(format!("{path}:type={kind}"));
    lines.push(format!("{path}:span={}", ref_span(node)));

    if let Some(name) = node["name"].as_str() {
        lines.push(format!("{path}:name={name:?}"));
    }

    if let Some(raw) = node["raw"].as_str() {
        lines.push(format!("{path}:raw={raw:?}"));
    }

    if let Some(test) = node.get("test") {
        lines.push(format!("{path}:test={:?}", ref_expr_source(test, source)));
    }
    if let Some(expression) = node.get("expression") {
        lines.push(format!(
            "{path}:expr={:?}",
            ref_expr_source(expression, source)
        ));
    }

    if let Some(attributes) = node["attributes"].as_array() {
        let attrs: Vec<String> = attributes
            .iter()
            .map(|attr| {
                format!(
                    "{}:{}",
                    attr["type"].as_str().unwrap_or(""),
                    attr["name"].as_str().unwrap_or("")
                )
            })
            .collect();
        lines.push(format!("{path}:attrs={attrs:?}"));
    }

    if let Some(fragment) = node.get("fragment") {
        emit_reference_fragment(lines, fragment, source, &format!("{path}/fragment"));
    }
    if let Some(body) = node.get("body") {
        emit_reference_fragment(lines, body, source, &format!("{path}/body"));
    }
    if let Some(consequent) = node.get("consequent") {
        emit_reference_fragment(lines, consequent, source, &format!("{path}/consequent"));
    }
    if !node["alternate"].is_null() {
        emit_reference_fragment(
            lines,
            &node["alternate"],
            source,
            &format!("{path}/alternate"),
        );
    }
    if !node["then"].is_null() {
        emit_reference_fragment(lines, &node["then"], source, &format!("{path}/then"));
    }
    if !node["catch"].is_null() {
        emit_reference_fragment(lines, &node["catch"], source, &format!("{path}/catch"));
    }
}

fn emit_lux_root(
    lines: &mut Vec<String>,
    parsed: &lux_parser::ParseResult<'_>,
    source: &str,
    path: &str,
) {
    lines.push(format!("{path}:errors={}", parsed.errors.len()));
    lines.push(format!("{path}:warnings={}", parsed.warnings.len()));

    let root = &parsed.root;
    lines.push(format!("{path}:options_present={}", root.options.is_some()));
    lines.push(format!(
        "{path}:instance_present={}",
        root.instance.is_some()
    ));
    lines.push(format!("{path}:module_present={}", root.module.is_some()));
    lines.push(format!("{path}:css_present={}", root.css.is_some()));
    lines.push(format!("{path}:fragment_len={}", root.fragment.nodes.len()));

    if let Some(options) = &root.options {
        lines.push(format!(
            "{path}/options:runes={}",
            options
                .runes
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".into())
        ));
        lines.push(format!(
            "{path}/options:namespace={}",
            options
                .namespace
                .map(|value| format!("{value:?}"))
                .unwrap_or_else(|| "none".into())
        ));
        lines.push(format!(
            "{path}/options:css={}",
            options
                .css
                .map(|value| format!("{value:?}"))
                .unwrap_or_else(|| "none".into())
        ));
        lines.push(format!(
            "{path}/options:custom_tag={}",
            options
                .custom_element
                .as_ref()
                .and_then(|custom| custom.tag)
                .map(|tag| tag.to_string())
                .unwrap_or_else(|| "none".into())
        ));
    } else {
        lines.push(format!("{path}/options:none"));
    }

    emit_lux_script(lines, root.instance.as_ref(), &format!("{path}/instance"));
    emit_lux_script(lines, root.module.as_ref(), &format!("{path}/module"));
    emit_lux_css(lines, root.css.as_ref(), &format!("{path}/css"));
    emit_lux_fragment(
        lines,
        &root.fragment.nodes,
        source,
        &format!("{path}/fragment"),
    );
}

fn emit_lux_script(
    lines: &mut Vec<String>,
    script: Option<&lux_ast::template::root::Script<'_>>,
    path: &str,
) {
    let Some(script) = script else {
        lines.push(format!("{path}:none"));
        return;
    };

    let context = match script.context {
        lux_ast::template::root::ScriptContext::Default => "default",
        lux_ast::template::root::ScriptContext::Module => "module",
    };
    lines.push(format!("{path}:context={context}"));
    let attrs: Vec<String> = script
        .attributes
        .iter()
        .map(|attr| attr.name.to_string())
        .collect();
    lines.push(format!("{path}:attrs={attrs:?}"));
}

fn emit_lux_css(lines: &mut Vec<String>, css: Option<&lux_ast::css::StyleSheet<'_>>, path: &str) {
    let Some(css) = css else {
        lines.push(format!("{path}:none"));
        return;
    };

    let children: Vec<&str> = css
        .children
        .iter()
        .map(|child| match child {
            lux_ast::css::stylesheet::StyleSheetChild::Rule(_) => "Rule",
            lux_ast::css::stylesheet::StyleSheetChild::Atrule(_) => "Atrule",
        })
        .collect();
    lines.push(format!("{path}:children={children:?}"));
}

fn emit_lux_fragment(
    lines: &mut Vec<String>,
    nodes: &[FragmentNode<'_>],
    source: &str,
    path: &str,
) {
    lines.push(format!("{path}:len={}", nodes.len()));
    for (index, node) in nodes.iter().enumerate() {
        emit_lux_node(lines, node, source, &format!("{path}/{index}"));
    }
}

fn emit_lux_node(lines: &mut Vec<String>, node: &FragmentNode<'_>, source: &str, path: &str) {
    match node {
        FragmentNode::Text(node) => {
            lines.push(format!("{path}:type=Text"));
            lines.push(format!(
                "{path}:span={}..{}",
                node.span.start, node.span.end
            ));
            lines.push(format!("{path}:raw={:?}", node.raw));
        }
        FragmentNode::ExpressionTag(node) => {
            lines.push(format!("{path}:type=ExpressionTag"));
            lines.push(format!(
                "{path}:span={}..{}",
                node.span.start, node.span.end
            ));
            lines.push(format!(
                "{path}:expr={:?}",
                slice(
                    source,
                    node.expression.span().start,
                    node.expression.span().end
                )
            ));
        }
        FragmentNode::HtmlTag(node) => {
            lines.push(format!("{path}:type=HtmlTag"));
            lines.push(format!(
                "{path}:span={}..{}",
                node.span.start, node.span.end
            ));
            lines.push(format!(
                "{path}:expr={:?}",
                slice(
                    source,
                    node.expression.span().start,
                    node.expression.span().end
                )
            ));
        }
        FragmentNode::ConstTag(node) => {
            lines.push(format!("{path}:type=ConstTag"));
            lines.push(format!(
                "{path}:span={}..{}",
                node.span.start, node.span.end
            ));
            lines.push(format!(
                "{path}:expr={:?}",
                slice(
                    source,
                    node.declaration.init.span().start,
                    node.declaration.init.span().end
                )
            ));
        }
        FragmentNode::DebugTag(node) => {
            lines.push(format!("{path}:type=DebugTag"));
            lines.push(format!(
                "{path}:span={}..{}",
                node.span.start, node.span.end
            ));
        }
        FragmentNode::RenderTag(node) => {
            lines.push(format!("{path}:type=RenderTag"));
            lines.push(format!(
                "{path}:span={}..{}",
                node.span.start, node.span.end
            ));
            lines.push(format!(
                "{path}:expr={:?}",
                slice(
                    source,
                    node.expression.span().start,
                    node.expression.span().end
                )
            ));
        }
        FragmentNode::Comment(node) => {
            lines.push(format!("{path}:type=Comment"));
            lines.push(format!(
                "{path}:span={}..{}",
                node.span.start, node.span.end
            ));
        }
        FragmentNode::IfBlock(node) => emit_lux_if(lines, node, source, path),
        FragmentNode::EachBlock(node) => emit_lux_each(lines, node, source, path),
        FragmentNode::AwaitBlock(node) => emit_lux_await(lines, node, source, path),
        FragmentNode::KeyBlock(node) => {
            lines.push(format!("{path}:type=KeyBlock"));
            lines.push(format!(
                "{path}:span={}..{}",
                node.span.start, node.span.end
            ));
            lines.push(format!(
                "{path}:expr={:?}",
                slice(
                    source,
                    node.expression.span().start,
                    node.expression.span().end
                )
            ));
            emit_lux_fragment(
                lines,
                &node.fragment.nodes,
                source,
                &format!("{path}/fragment"),
            );
        }
        FragmentNode::SnippetBlock(node) => {
            lines.push(format!("{path}:type=SnippetBlock"));
            lines.push(format!(
                "{path}:span={}..{}",
                node.span.start, node.span.end
            ));
            lines.push(format!("{path}:name={:?}", node.expression.name.as_str()));
            emit_lux_fragment(lines, &node.body.nodes, source, &format!("{path}/body"));
        }
        FragmentNode::RegularElement(node) => emit_lux_element(
            lines,
            "RegularElement",
            node.name,
            node.span.start,
            node.span.end,
            &node.attributes,
            &node.fragment.nodes,
            source,
            path,
        ),
        FragmentNode::Component(node) => emit_lux_element(
            lines,
            "Component",
            node.name,
            node.span.start,
            node.span.end,
            &node.attributes,
            &node.fragment.nodes,
            source,
            path,
        ),
        FragmentNode::SvelteComponent(node) => emit_lux_element(
            lines,
            "SvelteComponent",
            node.name,
            node.span.start,
            node.span.end,
            &node.attributes,
            &node.fragment.nodes,
            source,
            path,
        ),
        FragmentNode::SvelteElement(node) => emit_lux_element(
            lines,
            "SvelteElement",
            node.name,
            node.span.start,
            node.span.end,
            &node.attributes,
            &node.fragment.nodes,
            source,
            path,
        ),
        FragmentNode::SvelteSelf(node) => emit_lux_element(
            lines,
            "SvelteSelf",
            node.name,
            node.span.start,
            node.span.end,
            &node.attributes,
            &node.fragment.nodes,
            source,
            path,
        ),
        FragmentNode::SvelteFragment(node) => emit_lux_element(
            lines,
            "SvelteFragment",
            node.name,
            node.span.start,
            node.span.end,
            &node.attributes,
            &node.fragment.nodes,
            source,
            path,
        ),
        FragmentNode::SvelteHead(node) => emit_lux_element(
            lines,
            "SvelteHead",
            node.name,
            node.span.start,
            node.span.end,
            &node.attributes,
            &node.fragment.nodes,
            source,
            path,
        ),
        FragmentNode::SvelteBody(node) => emit_lux_element(
            lines,
            "SvelteBody",
            node.name,
            node.span.start,
            node.span.end,
            &node.attributes,
            &node.fragment.nodes,
            source,
            path,
        ),
        FragmentNode::SvelteWindow(node) => emit_lux_element(
            lines,
            "SvelteWindow",
            node.name,
            node.span.start,
            node.span.end,
            &node.attributes,
            &node.fragment.nodes,
            source,
            path,
        ),
        FragmentNode::SvelteDocument(node) => emit_lux_element(
            lines,
            "SvelteDocument",
            node.name,
            node.span.start,
            node.span.end,
            &node.attributes,
            &node.fragment.nodes,
            source,
            path,
        ),
        FragmentNode::SvelteBoundary(node) => emit_lux_element(
            lines,
            "SvelteBoundary",
            node.name,
            node.span.start,
            node.span.end,
            &node.attributes,
            &node.fragment.nodes,
            source,
            path,
        ),
        FragmentNode::SlotElement(node) => emit_lux_element(
            lines,
            "SlotElement",
            node.name,
            node.span.start,
            node.span.end,
            &node.attributes,
            &node.fragment.nodes,
            source,
            path,
        ),
        FragmentNode::TitleElement(node) => emit_lux_element(
            lines,
            "TitleElement",
            node.name,
            node.span.start,
            node.span.end,
            &node.attributes,
            &node.fragment.nodes,
            source,
            path,
        ),
        FragmentNode::AttachTag(node) => {
            lines.push(format!("{path}:type=AttachTag"));
            lines.push(format!(
                "{path}:span={}..{}",
                node.span.start, node.span.end
            ));
            lines.push(format!(
                "{path}:expr={:?}",
                slice(
                    source,
                    node.expression.span().start,
                    node.expression.span().end
                )
            ));
        }
        FragmentNode::SvelteOptionsRaw(node) => emit_lux_element(
            lines,
            "SvelteOptionsRaw",
            node.name,
            node.span.start,
            node.span.end,
            &node.attributes,
            &node.fragment.nodes,
            source,
            path,
        ),
    }
}

fn emit_lux_if(
    lines: &mut Vec<String>,
    node: &lux_ast::template::block::IfBlock<'_>,
    source: &str,
    path: &str,
) {
    lines.push(format!("{path}:type=IfBlock"));
    lines.push(format!(
        "{path}:span={}..{}",
        node.span.start, node.span.end
    ));
    lines.push(format!(
        "{path}:test={:?}",
        slice(source, node.test.span().start, node.test.span().end)
    ));
    emit_lux_fragment(
        lines,
        &node.consequent.nodes,
        source,
        &format!("{path}/consequent"),
    );
    if let Some(alt) = &node.alternate {
        emit_lux_fragment(lines, &alt.nodes, source, &format!("{path}/alternate"));
    }
}

fn emit_lux_each(
    lines: &mut Vec<String>,
    node: &lux_ast::template::block::EachBlock<'_>,
    source: &str,
    path: &str,
) {
    lines.push(format!("{path}:type=EachBlock"));
    lines.push(format!(
        "{path}:span={}..{}",
        node.span.start, node.span.end
    ));
    lines.push(format!(
        "{path}:expr={:?}",
        slice(
            source,
            node.expression.span().start,
            node.expression.span().end
        )
    ));
    emit_lux_fragment(lines, &node.body.nodes, source, &format!("{path}/body"));
}

fn emit_lux_await(
    lines: &mut Vec<String>,
    node: &lux_ast::template::block::AwaitBlock<'_>,
    source: &str,
    path: &str,
) {
    lines.push(format!("{path}:type=AwaitBlock"));
    lines.push(format!(
        "{path}:span={}..{}",
        node.span.start, node.span.end
    ));
    lines.push(format!(
        "{path}:expr={:?}",
        slice(
            source,
            node.expression.span().start,
            node.expression.span().end
        )
    ));
    if let Some(then_frag) = &node.then {
        emit_lux_fragment(lines, &then_frag.nodes, source, &format!("{path}/then"));
    }
    if let Some(catch_frag) = &node.catch {
        emit_lux_fragment(lines, &catch_frag.nodes, source, &format!("{path}/catch"));
    }
}

fn emit_lux_element(
    lines: &mut Vec<String>,
    kind: &str,
    name: &str,
    start: u32,
    end: u32,
    attributes: &[AttributeNode<'_>],
    children: &[FragmentNode<'_>],
    source: &str,
    path: &str,
) {
    lines.push(format!("{path}:type={kind}"));
    lines.push(format!("{path}:span={start}..{end}"));
    lines.push(format!("{path}:name={name:?}"));
    let attrs: Vec<String> = attributes.iter().map(norm_attr_kind).collect();
    lines.push(format!("{path}:attrs={attrs:?}"));
    emit_lux_fragment(lines, children, source, &format!("{path}/fragment"));
}

fn norm_attr_kind(attribute: &AttributeNode<'_>) -> String {
    match attribute {
        AttributeNode::Attribute(attribute) => format!("Attribute:{}", attribute.name),
        AttributeNode::SpreadAttribute(_) => "SpreadAttribute".into(),
        AttributeNode::BindDirective(d) => format!("BindDirective:{}", d.name),
        AttributeNode::ClassDirective(d) => format!("ClassDirective:{}", d.name),
        AttributeNode::StyleDirective(d) => format!("StyleDirective:{}", d.name),
        AttributeNode::OnDirective(d) => format!("OnDirective:{}", d.name),
        AttributeNode::TransitionDirective(d) => format!("TransitionDirective:{}", d.name),
        AttributeNode::AnimateDirective(d) => format!("AnimateDirective:{}", d.name),
        AttributeNode::UseDirective(d) => format!("UseDirective:{}", d.name),
        AttributeNode::LetDirective(d) => format!("LetDirective:{}", d.name),
        AttributeNode::AttachTag(_) => "AttachTag".into(),
    }
}

fn ref_span(node: &Value) -> String {
    let start = node["start"].as_u64().unwrap_or(0);
    let end = node["end"].as_u64().unwrap_or(0);
    format!("{start}..{end}")
}

fn ref_expr_source(expr: &Value, source: &str) -> String {
    let start = expr["start"].as_u64().unwrap_or(0) as usize;
    let end = expr["end"].as_u64().unwrap_or(0) as usize;
    source.get(start..end).unwrap_or("").to_string()
}

fn slice(source: &str, start: u32, end: u32) -> String {
    source
        .get(start as usize..end as usize)
        .unwrap_or("")
        .to_string()
}

fn opt_str(value: &Value) -> String {
    value
        .as_str()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "none".to_string())
}

fn opt_bool_str(value: &Value) -> String {
    value
        .as_bool()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}
