use lux_ast::analysis::AnalysisSeverity;
use napi::{Error, Result, Status};
use napi_derive::napi;
use oxc_allocator::Allocator;

#[napi(object)]
pub struct CompileOptions {
    pub ts: Option<bool>,
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
    pub css: Option<String>,
    pub css_hash: Option<String>,
    pub css_scope: Option<String>,
    pub runtime_modules: Vec<RuntimeModule>,
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
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
    let allocator = Allocator::default();
    let parse_result = lux_parser::parse(
        source,
        &allocator,
        options.and_then(|o| o.ts).unwrap_or(false),
    );
    let analysis = lux_analyzer::analyze(&parse_result.root);
    let transform = lux_transformer::transform(&parse_result.root, &analysis);

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

    let runtime_modules = transform
        .runtime_modules
        .into_iter()
        .map(|module| RuntimeModule {
            specifier: module.specifier,
            code: module.code,
        })
        .collect::<Vec<_>>();

    CompileOutput {
        js: transform.js,
        css: transform.css,
        css_hash: transform.css_hash,
        css_scope: transform.css_scope,
        runtime_modules,
        errors,
        warnings,
        ts: parse_result.root.ts,
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
        let output = compile_internal("<p>{name}</p>", Some(&CompileOptions { ts: Some(false) }));
        assert!(output.errors.is_empty());
        assert_eq!(output.runtime_modules.len(), 1);
        assert_eq!(output.runtime_modules[0].specifier, "lux/runtime/server");
    }
}
