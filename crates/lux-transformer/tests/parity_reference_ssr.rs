use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use lux_analyzer::analyze;
use lux_ast::common::Span;
use lux_ast::template::root::{CssOption, CustomElementOptions, Root, SvelteOptions};
use lux_parser::parse;
use lux_test_support::{
    ensure_svelte_runner, is_legacy_reference_sample, normalize_text, read_json, reference_root,
    run_node_script, workspace_root_from_manifest_dir,
};
use lux_transformer::{RuntimeModule, transform_with_filename};
use oxc_allocator::Allocator;
use rustc_hash::FxHashMap;
use serde_json::Value;
use serde_json::json;

#[derive(Debug, Clone)]
struct SsrCase {
    name: String,
    config: SampleConfig,
}

#[derive(Debug)]
struct RenderedOutput {
    body: String,
    head: String,
}

#[derive(Debug, Clone)]
struct SampleConfig {
    props: Value,
    id_prefix: Option<String>,
    csp: Option<Value>,
    mode: Vec<String>,
    error: Option<String>,
    compile_options: Value,
    without_normalize_html: bool,
}

impl Default for SampleConfig {
    fn default() -> Self {
        Self {
            props: Value::Object(Default::default()),
            id_prefix: None,
            csp: None,
            mode: Vec::new(),
            error: None,
            compile_options: Value::Object(Default::default()),
            without_normalize_html: false,
        }
    }
}

impl SampleConfig {
    fn supports_sync_render(&self, sample_name: &str, sample_dir: &Path) -> bool {
        if is_legacy_only_ssr_sample(sample_name) {
            return false;
        }
        if sample_name.starts_with("async-") || sample_name.contains("-async") {
            return false;
        }
        if !self.mode.is_empty() && !self.mode.iter().any(|mode| mode == "sync") {
            return false;
        }
        if sample_contains_async_await(sample_dir) {
            return false;
        }
        self.error.is_none()
    }

    fn preserve_comments(&self) -> bool {
        self.compile_options
            .get("preserveComments")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    fn as_json(&self) -> Value {
        json!({
            "props": self.props,
            "id_prefix": self.id_prefix,
            "csp": self.csp,
            "mode": self.mode,
            "error": self.error,
            "compile_options": self.compile_options,
            "without_normalize_html": self.without_normalize_html,
        })
    }

    fn render_config_json(&self) -> Value {
        json!({
            "props": self.props,
            "id_prefix": self.id_prefix,
            "csp": self.csp,
        })
    }
}

#[test]
fn parity_reference_ssr_render_subset() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = workspace_root_from_manifest_dir(&manifest_dir);
    let runner_dir = ensure_svelte_runner(&workspace_root);
    let samples_dir = reference_root(&workspace_root).join("server-side-rendering/samples");
    assert!(samples_dir.exists(), "missing {}", samples_dir.display());

    let cases = discover_ssr_cases(&runner_dir, &samples_dir);
    assert!(
        !cases.is_empty(),
        "no SSR parity cases selected from {}",
        samples_dir.display()
    );

    for case in cases {
        let sample_dir = samples_dir.join(&case.name);
        let main_path = sample_dir.join("main.svelte");
        assert!(main_path.exists(), "missing {}", main_path.display());

        let expected_body = read_optional_text(sample_dir.join("_expected.html"));
        let expected_head = read_optional_text(sample_dir.join("_expected_head.html"));

        let reference =
            render_reference_component(&runner_dir, &case.name, &sample_dir, &case.config);

        let actual = render_lux_component(&runner_dir, &case.name, &sample_dir, &case.config);
        assert_eq!(
            normalize_rendered_html(&actual.body, &case.config),
            normalize_rendered_html(&reference.body, &case.config),
            "lux body mismatch for `{}`\nreference:\n{}\nexpected fixture:\n{}\nactual:\n{}",
            case.name,
            reference.body,
            expected_body,
            actual.body
        );
        assert_eq!(
            normalize_rendered_html(&actual.head, &case.config),
            normalize_rendered_html(&reference.head, &case.config),
            "lux head mismatch for `{}`\nreference:\n{}\nexpected fixture:\n{}\nactual:\n{}",
            case.name,
            reference.head,
            expected_head,
            actual.head
        );
    }
}

#[test]
fn lux_ssr_preserves_nested_context_without_leaking_to_siblings() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = workspace_root_from_manifest_dir(&manifest_dir);
    let runner_dir = ensure_svelte_runner(&workspace_root);
    let sample_dir = create_local_context_sample(&workspace_root);
    let sample_name = unique_local_sample_name("context-nested");
    let config = SampleConfig::default();

    let output_dir = prepare_render_dir(&runner_dir, &sample_name, "lux");
    let (main_module_path, runtime_modules) =
        compile_lux_sample_dir(&sample_dir, &output_dir, &config);
    write_lux_runtime_package(&output_dir, &runtime_modules);
    let rendered = render_compiled_module(&output_dir, &main_module_path, &config, &runner_dir);

    assert_eq!(
        normalize_rendered_html(&rendered.body, &config),
        normalize_rendered_html("<p>ok</p> <p>false</p>", &config),
        "nested component render should inherit context without leaking it to siblings"
    );
    assert_eq!(normalize_rendered_html(&rendered.head, &config), "");

    let _ = fs::remove_dir_all(sample_dir);
}

fn discover_ssr_cases(runner_dir: &Path, samples_dir: &Path) -> Vec<SsrCase> {
    const DEFAULT_CASES: &[&str] = &[
        "attribute-boolean",
        "attribute-dynamic",
        "attribute-escape-quotes-spread-2",
        "attribute-static",
        "bindings-group",
        "comment",
        "comment-preserve",
        "component",
        "component-data-dynamic",
        "component-data-empty",
        "component-with-different-extension",
        "computed",
        "css",
        "css-injected-options",
        "css-injected-options-minify",
        "css-injected-options-nested",
        "default-data",
        "default-data-override",
        "directives",
        "dynamic-text",
        "dynamic-text-escaped",
        "dynamic-element-string",
        "dynamic-element-variable",
        "each-block",
        "empty-elements-closed",
        "entities",
        "helpers",
        "head-component-props-id",
        "head-html-and-component",
        "head-title",
        "if-block-false",
        "if-block-true",
        "option-scoped-class",
        "raw-mustaches",
        "sanitize-name",
        "static-div",
        "static-text",
        "spread-attributes",
        "textarea-value",
        "triple",
    ];

    let include_all = std::env::var_os("LUX_SSR_PARITY_ALL").is_some();
    let filter = std::env::var("LUX_SSR_SAMPLE_FILTER").ok();
    let default_case_names = DEFAULT_CASES
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let mut cases = Vec::new();
    let entries = fs::read_dir(samples_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", samples_dir.display()));

    for entry in entries.filter_map(Result::ok) {
        let sample_dir = entry.path();
        if !sample_dir.is_dir() {
            continue;
        }
        let Some(sample_name) = sample_dir.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if is_legacy_reference_sample(sample_name) {
            continue;
        }
        let config = load_sample_config(runner_dir, &sample_dir, sample_name);
        if !config.supports_sync_render(sample_name, &sample_dir) {
            continue;
        }
        if let Some(filter) = filter.as_deref() {
            if !sample_name.contains(filter) {
                continue;
            }
        } else if !include_all && !default_case_names.contains(sample_name) {
            continue;
        }
        cases.push(SsrCase {
            name: sample_name.to_string(),
            config,
        });
    }

    cases.sort_by(|left, right| left.name.cmp(&right.name));
    cases
}

fn sample_contains_async_await(sample_dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(sample_dir) else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("svelte") {
            return false;
        }
        fs::read_to_string(&path)
            .map(|source| source.contains("await "))
            .unwrap_or(false)
    })
}

fn is_legacy_only_ssr_sample(sample_name: &str) -> bool {
    matches!(sample_name, "component-binding" | "component-binding-renamed")
        || is_temporarily_unsupported_modern_ssr_sample(sample_name)
}

fn is_temporarily_unsupported_modern_ssr_sample(sample_name: &str) -> bool {
    matches!(
        sample_name,
        "select-value-component" | "select-value-implicit-value-complex"
    )
}

fn load_sample_config(runner_dir: &Path, sample_dir: &Path, sample_name: &str) -> SampleConfig {
    let config_path = sample_dir.join("_config.js");
    if !config_path.exists() {
        return SampleConfig::default();
    }

    let output_dir = runner_dir.join(".parity-server-render").join(sample_name);
    let _ = fs::create_dir_all(&output_dir);
    let output_path = output_dir.join("config.json");
    let run = run_node_script(
        runner_dir,
        "read_server_sample_config.mjs",
        [&config_path, &output_path],
    );
    assert!(
        run.status.success(),
        "config extraction failed for {}\nstdout:\n{}\nstderr:\n{}",
        config_path.display(),
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );

    let config = read_json(&output_path);
    SampleConfig {
        props: config
            .get("props")
            .cloned()
            .unwrap_or_else(|| Value::Object(Default::default())),
        id_prefix: config
            .get("id_prefix")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        csp: config.get("csp").cloned().filter(|value| !value.is_null()),
        mode: config
            .get("mode")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        error: config
            .get("error")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        compile_options: config
            .get("compile_options")
            .cloned()
            .unwrap_or_else(|| Value::Object(Default::default())),
        without_normalize_html: config
            .get("without_normalize_html")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    }
}

fn render_reference_component(
    runner_dir: &Path,
    sample_name: &str,
    sample_dir: &Path,
    config: &SampleConfig,
) -> RenderedOutput {
    let output_dir = prepare_render_dir(runner_dir, sample_name, "reference");
    let main_module_path =
        compile_reference_sample_dir(runner_dir, sample_dir, &output_dir, config);
    render_compiled_module(&output_dir, &main_module_path, config, runner_dir)
}

fn render_lux_component(
    runner_dir: &Path,
    sample_name: &str,
    sample_dir: &Path,
    config: &SampleConfig,
) -> RenderedOutput {
    let output_dir = prepare_render_dir(runner_dir, sample_name, "lux");
    let (main_module_path, runtime_modules) =
        compile_lux_sample_dir(sample_dir, &output_dir, config);
    write_lux_runtime_package(&output_dir, &runtime_modules);
    render_compiled_module(&output_dir, &main_module_path, config, runner_dir)
}

fn render_compiled_module(
    output_dir: &Path,
    module_path: &Path,
    config: &SampleConfig,
    runner_dir: &Path,
) -> RenderedOutput {
    let render_config_path = output_dir.join("render-config.json");
    let render_output_path = output_dir.join("render.json");

    write_json_file(&render_config_path, &config.render_config_json());

    let render = run_node_script(
        runner_dir,
        "render_server_module.mjs",
        [module_path, &render_config_path, &render_output_path],
    );
    assert!(
        render.status.success(),
        "server render failed for {}\nstdout:\n{}\nstderr:\n{}",
        module_path.display(),
        String::from_utf8_lossy(&render.stdout),
        String::from_utf8_lossy(&render.stderr),
    );

    let rendered = read_json(&render_output_path);
    RenderedOutput {
        body: rendered
            .get("body")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        head: rendered
            .get("head")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
    }
}

fn compile_reference_sample_dir(
    runner_dir: &Path,
    sample_dir: &Path,
    output_dir: &Path,
    config: &SampleConfig,
) -> PathBuf {
    let source_files = collect_svelte_source_files(sample_dir);
    copy_sample_support_files(sample_dir, output_dir);
    let config_path = output_dir.join("sample-config.json");
    write_json_file(&config_path, &config.as_json());
    for input_path in &source_files {
        let compile_output_path =
            output_dir.join(module_cache_name(sample_dir, input_path, "reference.json"));
        let compile = run_node_script(
            runner_dir,
            "compile_server.mjs",
            [input_path, &compile_output_path, &config_path],
        );
        assert!(
            compile.status.success(),
            "reference compile failed for {}\nstdout:\n{}\nstderr:\n{}",
            input_path.display(),
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr),
        );

        let compiled = read_json(&compile_output_path);
        let js = compiled
            .get("js")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let module_path = module_output_path(output_dir, sample_dir, input_path);
        write_module_file(&module_path, rewrite_local_svelte_imports(&js));
    }

    output_dir.join("main.svelte.mjs")
}

fn compile_lux_sample_dir(
    sample_dir: &Path,
    output_dir: &Path,
    config: &SampleConfig,
) -> (PathBuf, Vec<RuntimeModule>) {
    let source_files = collect_svelte_source_files(sample_dir);
    copy_sample_support_files(sample_dir, output_dir);
    let mut runtime_modules_by_specifier: FxHashMap<String, RuntimeModule> = FxHashMap::default();

    for input_path in &source_files {
        let source = fs::read_to_string(input_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", input_path.display()));
        let allocator = Allocator::default();
        let mut parsed = parse(&source, &allocator, false);
        assert!(
            parsed.errors.is_empty(),
            "parse failed for `{}` with {} errors",
            input_path.display(),
            parsed.errors.len()
        );

        apply_sample_compile_options(&mut parsed.root, config);
        let analysis = analyze(&parsed.root);
        assert!(
            analysis.diagnostics.is_empty(),
            "analysis diagnostics for `{}`: {:?}",
            input_path.display(),
            analysis
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>()
        );

        let filename = input_path.to_string_lossy();
        let transformed = transform_with_filename(&parsed.root, &analysis, Some(filename.as_ref()));
        for module in transformed.runtime_modules {
            runtime_modules_by_specifier
                .entry(module.specifier.clone())
                .or_insert(module);
        }

        let module_path = module_output_path(output_dir, sample_dir, input_path);
        write_module_file(&module_path, rewrite_local_svelte_imports(&transformed.js));
    }

    (
        output_dir.join("main.svelte.mjs"),
        runtime_modules_by_specifier.into_values().collect(),
    )
}

fn write_json_file(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_string(value).expect("serialize JSON"))
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", path.display()));
}

fn apply_sample_compile_options(root: &mut Root<'_>, config: &SampleConfig) {
    let compile_options = &config.compile_options;
    if !compile_options.is_object() {
        return;
    }

    let target = root
        .options
        .get_or_insert_with(|| empty_svelte_options(root.span));

    if target.runes.is_none() {
        target.runes = option_bool(compile_options, "runes");
    }
    if target.immutable.is_none() {
        target.immutable = option_bool(compile_options, "immutable");
    }
    if target.accessors.is_none() {
        target.accessors = option_bool(compile_options, "accessors");
    }
    if target.preserve_whitespace.is_none() {
        target.preserve_whitespace = option_bool(compile_options, "preserveWhitespace");
    }
    if target.css.is_none() && option_string(compile_options, "css") == Some("injected") {
        target.css = Some(CssOption::Injected);
    }
    if target.custom_element.is_none()
        && option_bool(compile_options, "customElement") == Some(true)
    {
        target.custom_element = Some(CustomElementOptions {
            tag: None,
            shadow: None,
            props: None,
            extend: None,
        });
    }
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

fn option_bool(options: &Value, key: &str) -> Option<bool> {
    options.get(key).and_then(Value::as_bool)
}

fn option_string<'a>(options: &'a Value, key: &str) -> Option<&'a str> {
    options.get(key).and_then(Value::as_str)
}

fn prepare_render_dir(runner_dir: &Path, sample_name: &str, kind: &str) -> PathBuf {
    let output_dir = runner_dir
        .join(".parity-server-render")
        .join(sample_name)
        .join(kind);
    let _ = fs::remove_dir_all(&output_dir);
    let _ = fs::create_dir_all(&output_dir);
    output_dir
}

fn write_lux_runtime_package(output_dir: &Path, runtime_modules: &[RuntimeModule]) {
    let package_root = output_dir.join("node_modules").join("lux");
    let runtime_package_json = package_root.join("package.json");
    if let Some(parent) = runtime_package_json.parent() {
        let _ = fs::create_dir_all(parent);
    }

    fs::write(
        &runtime_package_json,
        r#"{"name":"lux","type":"module","exports":{"./runtime/server":"./runtime/server.js","./runtime/client":"./runtime/client.js"}}"#,
    )
    .unwrap_or_else(|error| panic!("failed to write {}: {error}", runtime_package_json.display()));

    for module in runtime_modules {
        let relative_specifier = module
            .specifier
            .strip_prefix("lux/")
            .unwrap_or(module.specifier.as_str());
        let mut output_path = package_root.clone();
        for segment in relative_specifier.split('/') {
            output_path.push(segment);
        }
        if output_path.extension().is_none() {
            output_path.set_extension("js");
        }
        if let Some(parent) = output_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&output_path, &module.code)
            .unwrap_or_else(|error| panic!("failed to write {}: {error}", output_path.display()));
    }
}

fn collect_svelte_source_files(sample_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_svelte_source_files_recursive(sample_dir, &mut files);
    files.sort();
    files
}

fn copy_sample_support_files(sample_dir: &Path, output_dir: &Path) {
    copy_sample_support_files_recursive(sample_dir, sample_dir, output_dir);
}

fn copy_sample_support_files_recursive(root: &Path, dir: &Path, output_dir: &Path) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()));
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            copy_sample_support_files_recursive(root, &path, output_dir);
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if path.extension().and_then(|value| value.to_str()) == Some("svelte")
            || file_name == "_config.js"
            || file_name.starts_with("_expected")
        {
            continue;
        }

        let relative_path = path
            .strip_prefix(root)
            .unwrap_or_else(|error| panic!("failed to relativize {}: {error}", path.display()));
        let target_path = output_dir.join(relative_path);
        if let Some(parent) = target_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::copy(&path, &target_path).unwrap_or_else(|error| {
            panic!(
                "failed to copy support file {} to {}: {error}",
                path.display(),
                target_path.display()
            )
        });
    }
}

fn collect_svelte_source_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()));
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_svelte_source_files_recursive(&path, files);
            continue;
        }
        if path.extension().and_then(|value| value.to_str()) == Some("svelte") {
            files.push(path);
        }
    }
}

fn module_output_path(output_dir: &Path, sample_dir: &Path, input_path: &Path) -> PathBuf {
    let relative_path = input_path
        .strip_prefix(sample_dir)
        .unwrap_or_else(|error| panic!("failed to strip prefix {}: {error}", input_path.display()));
    let mut output_path = output_dir.join(relative_path);
    let file_name = output_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("main.svelte");
    output_path.set_file_name(format!("{file_name}.mjs"));
    output_path
}

fn module_cache_name(sample_dir: &Path, input_path: &Path, suffix: &str) -> String {
    input_path
        .strip_prefix(sample_dir)
        .expect("strip sample dir")
        .to_string_lossy()
        .replace(['\\', '/'], "__")
        + "."
        + suffix
}

fn write_module_file(module_path: &Path, js: String) {
    if let Some(parent) = module_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(module_path, js)
        .unwrap_or_else(|error| panic!("failed to write {}: {error}", module_path.display()));
}

fn rewrite_local_svelte_imports(js: &str) -> String {
    js.replace(".svelte\"", ".svelte.mjs\"")
        .replace(".svelte'", ".svelte.mjs'")
}

fn read_optional_text(path: PathBuf) -> String {
    fs::read_to_string(&path).unwrap_or_default()
}

fn normalize_rendered_html(input: &str, config: &SampleConfig) -> String {
    let with_normalized_newlines = input.replace('\r', "");
    let stripped_markers = strip_ssr_markers(with_normalized_newlines.trim());
    let without_comments = if config.preserve_comments() {
        stripped_markers
    } else {
        strip_html_comments(&stripped_markers)
    };

    if config.without_normalize_html {
        return normalize_text(&without_comments);
    }

    normalize_text(&collapse_whitespace(&normalize_void_elements(
        &without_comments,
    )))
}

fn strip_ssr_markers(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut rest = input.replace('\r', "");

    loop {
        let Some(start) = rest.find("<!--ssr:") else {
            output.push_str(&rest);
            break;
        };
        output.push_str(&rest[..start]);

        let marker = &rest[start..];
        let Some(end) = marker.find("-->") else {
            output.push_str(marker);
            break;
        };
        rest = marker[end + 3..].to_string();
    }

    output
}

fn strip_html_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut rest = input;

    loop {
        let Some(start) = rest.find("<!--") else {
            output.push_str(rest);
            break;
        };
        output.push_str(&rest[..start]);

        let comment = &rest[start..];
        let Some(end) = comment.find("-->") else {
            output.push_str(comment);
            break;
        };
        rest = &comment[end + 3..];
    }

    output
}

fn normalize_void_elements(input: &str) -> String {
    input.replace("/>", ">")
}

fn collapse_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn create_local_context_sample(workspace_root: &Path) -> PathBuf {
    let sample_dir = workspace_root
        .join("target")
        .join("lux-local-ssr")
        .join(unique_local_sample_name("context-sample"));
    let _ = fs::remove_dir_all(&sample_dir);
    fs::create_dir_all(&sample_dir)
        .unwrap_or_else(|error| panic!("failed to create {}: {error}", sample_dir.display()));

    fs::write(
        sample_dir.join("main.svelte"),
        r#"<script>
    import Provider from './Provider.svelte';
    import Optional from './Optional.svelte';
</script>

<Provider />
<Optional />
"#,
    )
    .unwrap_or_else(|error| panic!("failed to write main.svelte: {error}"));
    fs::write(
        sample_dir.join("Provider.svelte"),
        r#"<script>
    import { setContext } from 'svelte';
    import Child from './Child.svelte';

    setContext('answer', 'ok');
</script>

<Child />
"#,
    )
    .unwrap_or_else(|error| panic!("failed to write Provider.svelte: {error}"));
    fs::write(
        sample_dir.join("Child.svelte"),
        r#"<script>
    import { getContext } from 'svelte';

    const value = getContext('answer');
</script>

<p>{value}</p>
"#,
    )
    .unwrap_or_else(|error| panic!("failed to write Child.svelte: {error}"));
    fs::write(
        sample_dir.join("Optional.svelte"),
        r#"<script>
    import { hasContext } from 'svelte';

    const has = hasContext('answer');
</script>

<p>{has}</p>
"#,
    )
    .unwrap_or_else(|error| panic!("failed to write Optional.svelte: {error}"));

    sample_dir
}

fn unique_local_sample_name(prefix: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after epoch");
    format!("{prefix}-{}-{}", std::process::id(), now.as_nanos())
}
