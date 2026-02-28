use std::collections::HashSet;

use lux_ast::analysis::{
    AnalysisDiagnostic, AnalysisDiagnosticCode, AnalysisSeverity, AnalysisTables, TemplateScopeId,
};
use oxc_syntax::symbol::SymbolFlags;

#[derive(Debug, Default, Clone, Copy)]
struct ScriptWriteInfo {
    has_const: bool,
    has_import: bool,
}

pub(crate) fn emit_assignment_diagnostics(tables: &mut AnalysisTables) {
    let script_write_info = collect_script_write_info(tables);
    let mut pending = Vec::new();
    let mut seen = HashSet::new();

    for reference in tables
        .template_references
        .iter()
        .filter(|reference| reference.is_write)
    {
        let diagnostic =
            if resolve_template_binding(tables, reference.scope, &reference.name).is_some() {
                Some(AnalysisDiagnostic {
                    severity: AnalysisSeverity::Error,
                    code: AnalysisDiagnosticCode::TemplateAssignmentToBinding,
                    message: format!("Cannot assign to template binding `{}`", reference.name),
                    span: reference.span,
                })
            } else if let Some(write_info) = script_write_info.get(reference.name.as_str()) {
                if write_info.has_import {
                    Some(AnalysisDiagnostic {
                        severity: AnalysisSeverity::Error,
                        code: AnalysisDiagnosticCode::TemplateAssignmentToImport,
                        message: format!("Cannot assign to import `{}`", reference.name),
                        span: reference.span,
                    })
                } else if write_info.has_const {
                    Some(AnalysisDiagnostic {
                        severity: AnalysisSeverity::Error,
                        code: AnalysisDiagnosticCode::TemplateAssignmentToConst,
                        message: format!("Cannot assign to constant `{}`", reference.name),
                        span: reference.span,
                    })
                } else {
                    None
                }
            } else {
                None
            };

        if let Some(diagnostic) = diagnostic {
            let key = (diagnostic.code, diagnostic.span.start, diagnostic.span.end);
            if seen.insert(key) {
                pending.push(diagnostic);
            }
        }
    }

    tables.diagnostics.extend(pending);
}

fn resolve_template_binding(
    tables: &AnalysisTables,
    scope: TemplateScopeId,
    name: &str,
) -> Option<()> {
    let mut current_scope = Some(scope);

    while let Some(scope_id) = current_scope {
        if tables
            .template_bindings
            .iter()
            .any(|binding| binding.scope == scope_id && binding.name == name)
        {
            return Some(());
        }

        current_scope = tables
            .template_scopes
            .iter()
            .find(|candidate| candidate.id == scope_id)
            .and_then(|candidate| candidate.parent);
    }

    None
}

fn collect_script_write_info(
    tables: &AnalysisTables,
) -> std::collections::HashMap<&str, ScriptWriteInfo> {
    let mut info_map = std::collections::HashMap::new();

    for symbol in &tables.script_symbols {
        let symbol_flags = SymbolFlags::from_bits_truncate(symbol.flags);
        let entry = info_map
            .entry(symbol.name.as_str())
            .or_insert_with(ScriptWriteInfo::default);

        if symbol_flags.is_import() {
            entry.has_import = true;
        }

        if symbol_flags.is_const_variable() {
            entry.has_const = true;
        }
    }

    info_map
}
