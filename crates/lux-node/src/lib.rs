use std::path::Path;

use lux_ast::analysis::{AnalysisSeverity, ScriptRuneKind};
use lux_ast::common::Span;
use lux_ast::template::root::{CssOption, CustomElementOptions, Namespace, Root, SvelteOptions};
use lux_transformer::TransformTarget;
use napi::{Error, Result, Status};
use napi_derive::napi;
use oxc_allocator::Allocator;
use serde_json::json;

#[napi(object)]
#[derive(Default)]
pub struct CompileOptions {
    pub ts: Option<bool>,
    pub generate: Option<String>,
    pub filename: Option<String>,
    pub root_dir: Option<String>,
    pub runes: Option<bool>,
    pub immutable: Option<bool>,
    pub accessors: Option<bool>,
    pub preserve_whitespace: Option<bool>,
    pub custom_element: Option<bool>,
    pub css: Option<String>,
    pub output_filename: Option<String>,
    pub css_output_filename: Option<String>,
    pub modern_ast: Option<bool>,
}

#[napi(object)]
pub struct Diagnostic {
    pub phase: String,
    pub severity: String,
    pub code: Option<String>,
    pub message: String,
    pub start: u32,
    pub end: u32,
}

#[napi(object)]
pub struct RuntimeModule {
    pub specifier: String,
    pub code: String,
}

#[napi(object)]
pub struct CompileOutput {
    pub js: String,
    pub js_map: Option<String>,
    pub css: Option<String>,
    pub css_map: Option<String>,
    pub css_hash: Option<String>,
    pub css_scope: Option<String>,
    pub runtime_modules: Vec<RuntimeModule>,
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
    pub metadata_runes: bool,
    pub ast_json: Option<String>,
    pub ts: bool,
}

#[napi(js_name = "compile")]
pub fn compile_js(source: String, options: Option<CompileOptions>) -> CompileOutput {
    compile_internal(&source, options.as_ref())
}

#[napi(js_name = "compileStrict")]
pub fn compile_strict_js(source: String, options: Option<CompileOptions>) -> Result<CompileOutput> {
    let output = compile_internal(&source, options.as_ref());
    if output.errors.is_empty() {
        return Ok(output);
    }

    let first = &output.errors[0];
    Err(Error::new(
        Status::GenericFailure,
        format!(
            "{} [{}:{}-{}]",
            first.message, first.phase, first.start, first.end
        ),
    ))
}

fn compile_internal(source: &str, options: Option<&CompileOptions>) -> CompileOutput {
    let generate_target = options
        .and_then(|o| o.generate.as_deref())
        .unwrap_or("server");
    let allocator = Allocator::default();
    let mut parse_result = lux_parser::parse(
        source,
        &allocator,
        options.and_then(|o| o.ts).unwrap_or(false),
    );
    apply_compile_options_to_root(&mut parse_result.root, options);
    let analysis = lux_analyzer::analyze(&parse_result.root);
    let metadata_runes = effective_metadata_runes(&parse_result.root, &analysis);
    let ast_json = Some(build_ast_json(&parse_result.root));

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for parse_error in &parse_result.errors {
        errors.push(Diagnostic {
            phase: "parse".to_string(),
            severity: "error".to_string(),
            code: parse_error.code.map(ToString::to_string),
            message: parse_error.message.clone(),
            start: parse_error.span.start,
            end: parse_error.span.end,
        });
    }

    for parse_warning in &parse_result.warnings {
        warnings.push(Diagnostic {
            phase: "parse".to_string(),
            severity: "warning".to_string(),
            code: Some(parse_warning.code.to_string()),
            message: parse_warning.message.clone(),
            start: parse_warning.span.start,
            end: parse_warning.span.end,
        });
    }

    for diagnostic in &analysis.diagnostics {
        let item = Diagnostic {
            phase: "analyze".to_string(),
            severity: match diagnostic.severity {
                AnalysisSeverity::Error => "error".to_string(),
                AnalysisSeverity::Warning => "warning".to_string(),
            },
            code: Some(format!("{:?}", diagnostic.code)),
            message: diagnostic.message.clone(),
            start: diagnostic.span.start,
            end: diagnostic.span.end,
        };

        if matches!(diagnostic.severity, AnalysisSeverity::Error) {
            errors.push(item);
        } else {
            warnings.push(item);
        }
    }

    let transform_target = match generate_target {
        "server" => TransformTarget::Server,
        "client" => TransformTarget::Client,
        _ => {
            errors.push(Diagnostic {
                phase: "transform".to_string(),
                severity: "error".to_string(),
                code: Some("unsupported_generate_target".to_string()),
                message: format!(
                    "Lux transform target `{generate_target}` is unsupported; expected `server` or `client`"
                ),
                start: 0,
                end: 0,
            });

            return CompileOutput {
                js: String::new(),
                js_map: None,
                css: None,
                css_map: None,
                css_hash: None,
                css_scope: None,
                runtime_modules: Vec::new(),
                errors,
                warnings,
                metadata_runes,
                ast_json,
                ts: parse_result.root.ts,
            };
        }
    };

    let transform =
        lux_transformer::transform_for_target(&parse_result.root, &analysis, transform_target);
    let runtime_modules = transform
        .runtime_modules
        .into_iter()
        .map(|module| RuntimeModule {
            specifier: module.specifier,
            code: module.code,
        })
        .collect::<Vec<_>>();
    let source_filename = options.and_then(|o| o.filename.as_deref());
    let js_map = Some(build_placeholder_sourcemap_json(
        source,
        source_filename,
        options.and_then(|o| o.output_filename.as_deref()),
        "js",
    ));
    let css_map = transform.css.as_ref().map(|_| {
        build_placeholder_sourcemap_json(
            source,
            source_filename,
            options.and_then(|o| o.css_output_filename.as_deref()),
            "css",
        )
    });

    CompileOutput {
        js: transform.js,
        js_map,
        css: transform.css,
        css_map,
        css_hash: transform.css_hash,
        css_scope: transform.css_scope,
        runtime_modules,
        errors,
        warnings,
        metadata_runes,
        ast_json,
        ts: parse_result.root.ts,
    }
}

fn apply_compile_options_to_root(root: &mut Root<'_>, options: Option<&CompileOptions>) {
    let Some(options) = options else {
        return;
    };
    if !has_root_option_overrides(options) {
        return;
    }

    let root_span = root.span;
    let target = root
        .options
        .get_or_insert_with(|| empty_svelte_options(root_span));

    if target.runes.is_none() {
        target.runes = options.runes;
    }
    if target.immutable.is_none() {
        target.immutable = options.immutable;
    }
    if target.accessors.is_none() {
        target.accessors = options.accessors;
    }
    if target.preserve_whitespace.is_none() {
        target.preserve_whitespace = options.preserve_whitespace;
    }
    if target.css.is_none() {
        target.css = parse_css_option(options.css.as_deref());
    }
    if target.custom_element.is_none() && options.custom_element == Some(true) {
        target.custom_element = Some(CustomElementOptions {
            tag: None,
            shadow: None,
            props: None,
            extend: None,
        });
    }
}

fn has_root_option_overrides(options: &CompileOptions) -> bool {
    options.runes.is_some()
        || options.immutable.is_some()
        || options.accessors.is_some()
        || options.preserve_whitespace.is_some()
        || options.custom_element == Some(true)
        || parse_css_option(options.css.as_deref()).is_some()
}

fn empty_svelte_options(span: Span) -> SvelteOptions<'static> {
    SvelteOptions {
        span,
        runes: None,
        immutable: None,
        accessors: None,
        preserve_whitespace: None,
        namespace: None,
        css: None,
        custom_element: None,
        attributes: Vec::new(),
    }
}

fn parse_css_option(value: Option<&str>) -> Option<CssOption> {
    match value {
        Some("injected") => Some(CssOption::Injected),
        _ => None,
    }
}

fn effective_metadata_runes(root: &Root<'_>, analysis: &lux_ast::analysis::AnalysisTables) -> bool {
    root.options
        .as_ref()
        .and_then(|options| options.runes)
        .unwrap_or_else(|| {
            analysis
                .script_runes
                .iter()
                .any(|rune| rune.kind == ScriptRuneKind::Known)
        })
}

fn build_placeholder_sourcemap_json(
    source: &str,
    source_filename: Option<&str>,
    output_filename: Option<&str>,
    extension: &str,
) -> String {
    let input_name = source_filename.unwrap_or("Component.svelte");
    let file = output_filename
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| default_output_filename(input_name, extension));

    json!({
        "version": 3,
        "file": file,
        "sources": [input_name],
        "sourcesContent": [source],
        "names": [],
        "mappings": "",
    })
    .to_string()
}

fn default_output_filename(input_name: &str, extension: &str) -> String {
    let stem = Path::new(input_name)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("component");
    format!("{stem}.{extension}")
}

fn build_ast_json(root: &Root<'_>) -> String {
    let option_json = root.options.as_ref().map(|options| {
        json!({
            "runes": options.runes,
            "immutable": options.immutable,
            "accessors": options.accessors,
            "preserveWhitespace": options.preserve_whitespace,
            "namespace": options.namespace.map(namespace_name),
            "css": options.css.map(css_option_name),
            "customElement": options.custom_element.is_some(),
        })
    });
    let node_types = root
        .fragment
        .nodes
        .iter()
        .map(fragment_node_kind)
        .collect::<Vec<_>>();

    json!({
        "type": "Root",
        "start": root.span.start,
        "end": root.span.end,
        "options": option_json,
        "fragment": {
            "nodeTypes": node_types,
            "dynamic": root.fragment.dynamic,
            "transparent": root.fragment.transparent,
        },
        "css": root.css.is_some(),
        "instance": root.instance.is_some(),
        "module": root.module.is_some(),
        "ts": root.ts,
    })
    .to_string()
}

fn namespace_name(namespace: Namespace) -> &'static str {
    match namespace {
        Namespace::Html => "html",
        Namespace::Svg => "svg",
        Namespace::Mathml => "mathml",
    }
}

fn css_option_name(option: CssOption) -> &'static str {
    match option {
        CssOption::Injected => "injected",
    }
}

fn fragment_node_kind(node: &lux_ast::template::root::FragmentNode<'_>) -> &'static str {
    use lux_ast::template::root::FragmentNode;

    match node {
        FragmentNode::Text(_) => "Text",
        FragmentNode::ExpressionTag(_) => "ExpressionTag",
        FragmentNode::HtmlTag(_) => "HtmlTag",
        FragmentNode::ConstTag(_) => "ConstTag",
        FragmentNode::DebugTag(_) => "DebugTag",
        FragmentNode::RenderTag(_) => "RenderTag",
        FragmentNode::AttachTag(_) => "AttachTag",
        FragmentNode::Comment(_) => "Comment",
        FragmentNode::RegularElement(_) => "RegularElement",
        FragmentNode::Component(_) => "Component",
        FragmentNode::SvelteElement(_) => "SvelteElement",
        FragmentNode::SvelteComponent(_) => "SvelteComponent",
        FragmentNode::SvelteSelf(_) => "SvelteSelf",
        FragmentNode::SvelteFragment(_) => "SvelteFragment",
        FragmentNode::SvelteHead(_) => "SvelteHead",
        FragmentNode::SvelteBody(_) => "SvelteBody",
        FragmentNode::SvelteWindow(_) => "SvelteWindow",
        FragmentNode::SvelteDocument(_) => "SvelteDocument",
        FragmentNode::SvelteBoundary(_) => "SvelteBoundary",
        FragmentNode::SlotElement(_) => "SlotElement",
        FragmentNode::TitleElement(_) => "TitleElement",
        FragmentNode::SvelteOptionsRaw(_) => "SvelteOptionsRaw",
        FragmentNode::IfBlock(_) => "IfBlock",
        FragmentNode::EachBlock(_) => "EachBlock",
        FragmentNode::AwaitBlock(_) => "AwaitBlock",
        FragmentNode::KeyBlock(_) => "KeyBlock",
        FragmentNode::SnippetBlock(_) => "SnippetBlock",
    }
}

#[cfg(test)]
mod tests {
    use super::{CompileOptions, compile_internal};

    #[test]
    fn compile_collects_parse_errors() {
        let output = compile_internal("{#if x}<div>", None);
        assert!(!output.errors.is_empty());
        assert_eq!(output.errors[0].phase, "parse");
    }

    #[test]
    fn compile_emits_runtime_module_for_dynamic_expression() {
        let output = compile_internal(
            "<p>{name}</p>",
            Some(&CompileOptions {
                ts: Some(false),
                generate: None,
                ..CompileOptions::default()
            }),
        );
        assert!(output.errors.is_empty());
        assert_eq!(output.runtime_modules.len(), 1);
        assert_eq!(output.runtime_modules[0].specifier, "lux/runtime/server");
    }

    #[test]
    fn compile_respects_explicit_ts_option_without_lang_attribute() {
        let output = compile_internal(
            "<script>let count: number = 1;</script>{count}",
            Some(&CompileOptions {
                ts: Some(true),
                generate: None,
                ..CompileOptions::default()
            }),
        );
        assert!(
            output.errors.is_empty(),
            "expected no parse errors with explicit ts option, got {}",
            output.errors.len()
        );
    }

    #[test]
    fn compile_supports_client_generate_target() {
        let output = compile_internal(
            "<p>ok</p>",
            Some(&CompileOptions {
                ts: Some(false),
                generate: Some("client".to_string()),
                ..CompileOptions::default()
            }),
        );
        assert!(output.errors.is_empty());
        assert!(
            output.js.contains("from \"lux/runtime/client\";"),
            "client transform should import lux/runtime/client"
        );
        assert!(
            output
                .runtime_modules
                .iter()
                .any(|module| module.specifier == "lux/runtime/client")
        );
        assert!(output.js_map.is_some());
        assert_eq!(output.metadata_runes, false);
        assert!(output.ast_json.is_some());
    }

    #[test]
    fn compile_reports_unsupported_generate_target() {
        let output = compile_internal(
            "<p>ok</p>",
            Some(&CompileOptions {
                ts: Some(false),
                generate: Some("edge".to_string()),
                ..CompileOptions::default()
            }),
        );
        assert!(
            output
                .errors
                .iter()
                .any(|diagnostic| diagnostic.code.as_deref() == Some("unsupported_generate_target"))
        );
    }

    #[test]
    fn compile_applies_external_runes_and_custom_element_options() {
        let output = compile_internal(
            "<script>export let tag;</script><div />",
            Some(&CompileOptions {
                runes: Some(true),
                custom_element: Some(true),
                ..CompileOptions::default()
            }),
        );
        assert!(
            output
                .errors
                .iter()
                .any(|diagnostic| { diagnostic.code.as_deref() == Some("LegacyExportInvalid") })
        );
        assert!(output.metadata_runes);
    }

    #[test]
    fn compile_emits_placeholder_maps_with_filenames() {
        let output = compile_internal(
            "<style>h1{color:red}</style><h1>ok</h1>",
            Some(&CompileOptions {
                filename: Some("/src/App.svelte".to_string()),
                output_filename: Some("App.js".to_string()),
                css_output_filename: Some("App.css".to_string()),
                ..CompileOptions::default()
            }),
        );

        let js_map = output.js_map.expect("expected js map");
        assert!(js_map.contains("\"file\":\"App.js\""));
        assert!(js_map.contains("\"sources\":[\"/src/App.svelte\"]"));

        let css_map = output.css_map.expect("expected css map");
        assert!(css_map.contains("\"file\":\"App.css\""));
    }
}
