use rustc_hash::FxHashSet;

use lux_ast::analysis::{
    AnalysisTables, ScriptReferenceAnalysis, ScriptScopeAnalysis, ScriptSymbolAnalysis,
    ScriptTarget,
};
use oxc_ast::AstKind;
use oxc_ast::ast::Program;
use oxc_semantic::{ReferenceId, Semantic, SemanticBuilder};

pub(super) fn analyze_program(
    program: &Program<'_>,
    target: ScriptTarget,
    tables: &mut AnalysisTables,
) {
    let semantic_result = SemanticBuilder::new().build(program);
    let semantic = semantic_result.semantic;

    add_scope_records(&semantic, target, tables);
    add_symbol_records(&semantic, target, tables);
    add_reference_records(&semantic, target, tables);
}

fn add_scope_records(semantic: &Semantic<'_>, target: ScriptTarget, tables: &mut AnalysisTables) {
    let scoping = semantic.scoping();

    for scope_id in scoping.scope_descendants_from_root() {
        tables.script_scopes.push(ScriptScopeAnalysis {
            target,
            id: scope_id.index() as u32,
            parent: scoping
                .scope_parent_id(scope_id)
                .map(|parent| parent.index() as u32),
            flags: scoping.scope_flags(scope_id).bits(),
            node_id: scoping.get_node_id(scope_id).index() as u32,
        });
    }
}

fn add_symbol_records(semantic: &Semantic<'_>, target: ScriptTarget, tables: &mut AnalysisTables) {
    let scoping = semantic.scoping();

    for symbol_id in scoping.symbol_ids() {
        tables.script_symbols.push(ScriptSymbolAnalysis {
            target,
            id: symbol_id.index() as u32,
            name: scoping.symbol_name(symbol_id).to_owned(),
            scope_id: scoping.symbol_scope_id(symbol_id).index() as u32,
            declaration_node_id: scoping.symbol_declaration(symbol_id).index() as u32,
            declaration_span: scoping.symbol_span(symbol_id),
            flags: scoping.symbol_flags(symbol_id).bits(),
            mutated: scoping.symbol_is_mutated(symbol_id),
            unused: scoping.symbol_is_unused(symbol_id),
        });
    }
}

fn add_reference_records(
    semantic: &Semantic<'_>,
    target: ScriptTarget,
    tables: &mut AnalysisTables,
) {
    let scoping = semantic.scoping();
    let mut seen_reference_ids: FxHashSet<usize> = FxHashSet::default();

    for symbol_id in scoping.symbol_ids() {
        for &reference_id in scoping.get_resolved_reference_ids(symbol_id) {
            if seen_reference_ids.insert(reference_id.index()) {
                add_reference_record(semantic, target, reference_id, tables);
            }
        }
    }

    for unresolved_group in scoping.root_unresolved_references_ids() {
        for reference_id in unresolved_group {
            if seen_reference_ids.insert(reference_id.index()) {
                add_reference_record(semantic, target, reference_id, tables);
            }
        }
    }
}

fn add_reference_record(
    semantic: &Semantic<'_>,
    target: ScriptTarget,
    reference_id: ReferenceId,
    tables: &mut AnalysisTables,
) {
    let scoping = semantic.scoping();
    let reference = scoping.get_reference(reference_id);
    let node = semantic.nodes().get_node(reference.node_id());

    let AstKind::IdentifierReference(identifier) = node.kind() else {
        return;
    };

    tables.script_references.push(ScriptReferenceAnalysis {
        target,
        id: reference_id.index() as u32,
        name: identifier.name.as_str().to_owned(),
        span: identifier.span,
        scope_id: reference.scope_id().index() as u32,
        symbol_id: reference
            .symbol_id()
            .map(|symbol_id| symbol_id.index() as u32),
        is_read: reference.is_read(),
        is_write: reference.is_write(),
    });
}
