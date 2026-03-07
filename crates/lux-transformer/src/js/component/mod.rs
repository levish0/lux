mod consts;
mod exports;
mod script;

use std::collections::BTreeSet;

use lux_ast::analysis::{AnalysisTables, ScriptTarget};
use lux_ast::template::attribute::AttributeNode;
use lux_ast::template::root::FragmentNode;
use lux_ast::template::root::Root;
use oxc_allocator::Allocator;
use oxc_allocator::CloneIn;
use oxc_ast::AstBuilder;
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, Statement};
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_span::{SPAN, SourceType};

use self::consts::{
    LUX_ANIMATE_ATTR, LUX_ATTR, LUX_ATTRIBUTES, LUX_BEGIN_RENDER, LUX_BIND_ATTR,
    LUX_BIND_TARGET_ATTR, LUX_CLASS_ATTR, LUX_CLEANUP_MOUNT, LUX_CSS, LUX_CSS_HASH, LUX_CSS_SCOPE,
    LUX_END_RENDER, LUX_ESCAPE, LUX_ESCAPE_ATTR, LUX_EVENT_ATTR, LUX_EVENT_TARGET_ATTR,
    LUX_HAS_DYNAMIC, LUX_IS_BOOLEAN_ATTR, LUX_MOUNT_ACTIONS, LUX_MOUNT_ANIMATIONS,
    LUX_MOUNT_BINDINGS, LUX_MOUNT_EVENTS, LUX_MOUNT_HTML, LUX_MOUNT_TRANSITIONS, LUX_ONCE,
    LUX_PROPS_ID, LUX_RUNTIME_CLIENT_IMPORT_SOURCE, LUX_RUNTIME_SERVER_IMPORT_SOURCE,
    LUX_STRINGIFY, LUX_STYLE_ATTR, LUX_TEMPLATE, LUX_TRANSITION_ATTR, LUX_USE_ATTR,
    optional_string_expr, push_const,
};
use self::exports::{
    client_default_export_statement, default_export_statement, named_export_statement,
};
use self::script::{
    collect_instance_runtime_binding_names, collect_instance_runtime_statements,
    collect_module_runtime_statements,
};
use super::ComponentRenderOutput;
use super::template::{RuntimeScope, build_render_nodes_expression, render_nodes_template};
use crate::TransformTarget;

pub(super) fn render(
    root: &Root<'_>,
    analysis: &AnalysisTables,
    css: Option<&str>,
    css_hash: Option<&str>,
    css_scope: Option<&str>,
    target: TransformTarget,
) -> ComponentRenderOutput {
    let partition = partition_top_level_nodes(&root.fragment.nodes);
    let template_result = render_nodes_template(&partition.body_nodes);
    let head_result = render_nodes_template(&partition.head_nodes);
    let has_global_target_hooks = target == TransformTarget::Client
        && partition
            .body_nodes
            .iter()
            .any(|node| node_has_global_target_hooks(node));
    let needs_runtime =
        template_result.has_dynamic || head_result.has_dynamic || target == TransformTarget::Client;

    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);

    let mut body = ast.vec_with_capacity(16);
    if needs_runtime {
        push_runtime_helper_import(ast, &mut body, target);
    }
    push_import_declarations(ast, &mut body, root, analysis);
    let module_runtime = collect_module_runtime_statements(ast, root);
    body.extend(module_runtime);

    push_const(
        ast,
        &mut body,
        LUX_TEMPLATE,
        ast.expression_string_literal(SPAN, ast.atom(template_result.html.as_str()), None),
    );
    push_const(ast, &mut body, LUX_CSS, optional_string_expr(ast, css));
    push_const(
        ast,
        &mut body,
        LUX_CSS_HASH,
        optional_string_expr(ast, css_hash),
    );
    push_const(
        ast,
        &mut body,
        LUX_CSS_SCOPE,
        optional_string_expr(ast, css_scope),
    );
    push_const(
        ast,
        &mut body,
        LUX_HAS_DYNAMIC,
        ast.expression_boolean_literal(
            SPAN,
            template_result.has_dynamic || has_global_target_hooks,
        ),
    );

    body.push(named_export_statement(ast));
    let instance_runtime = collect_instance_runtime_statements(ast, root);
    let mut scope_names = collect_instance_import_names(analysis);
    scope_names.extend(collect_instance_runtime_binding_names(&instance_runtime));
    let scope = RuntimeScope::from_names(scope_names);
    let render_expression = if template_result.has_dynamic || has_global_target_hooks {
        build_render_nodes_expression(ast, &partition.body_nodes, &scope)
    } else {
        ast.expression_identifier(SPAN, ast.ident(LUX_TEMPLATE))
    };
    let has_head = !partition.head_nodes.is_empty();
    let head_expression = has_head.then(|| {
        if head_result.has_dynamic {
            build_render_nodes_expression(ast, &partition.head_nodes, &scope)
        } else {
            ast.expression_string_literal(SPAN, ast.atom(head_result.html.as_str()), None)
        }
    });
    match target {
        TransformTarget::Server => body.push(default_export_statement(
            ast,
            render_expression,
            instance_runtime,
            head_expression,
            has_head.then(|| collect_instance_runtime_statements(ast, root)),
        )),
        TransformTarget::Client => body.push(client_default_export_statement(
            ast,
            render_expression,
            instance_runtime,
            head_expression,
        )),
    }

    let program = ast.program(
        SPAN,
        SourceType::mjs(),
        "",
        ast.vec(),
        None,
        ast.vec(),
        body,
    );
    ComponentRenderOutput {
        js: Codegen::new().build(&program).code,
        needs_runtime,
    }
}

struct TopLevelNodePartition<'a> {
    body_nodes: Vec<&'a FragmentNode<'a>>,
    head_nodes: Vec<&'a FragmentNode<'a>>,
}

fn partition_top_level_nodes<'a>(nodes: &'a [FragmentNode<'a>]) -> TopLevelNodePartition<'a> {
    let mut body_nodes = Vec::new();
    let mut head_nodes = Vec::new();

    for node in nodes {
        match node {
            FragmentNode::SvelteHead(head) => {
                head_nodes.extend(head.fragment.nodes.iter());
            }
            _ => body_nodes.push(node),
        }
    }

    TopLevelNodePartition {
        body_nodes,
        head_nodes,
    }
}

fn node_has_global_target_hooks(node: &FragmentNode<'_>) -> bool {
    match node {
        FragmentNode::SvelteWindow(node) => {
            attributes_have_runtime_target_hooks(&node.attributes)
                || node.fragment.nodes.iter().any(node_has_global_target_hooks)
        }
        FragmentNode::SvelteDocument(node) => {
            attributes_have_runtime_target_hooks(&node.attributes)
                || node.fragment.nodes.iter().any(node_has_global_target_hooks)
        }
        FragmentNode::SvelteBody(node) => {
            attributes_have_runtime_target_hooks(&node.attributes)
                || node.fragment.nodes.iter().any(node_has_global_target_hooks)
        }
        FragmentNode::RegularElement(node) => {
            node.fragment.nodes.iter().any(node_has_global_target_hooks)
        }
        FragmentNode::TitleElement(node) => {
            node.fragment.nodes.iter().any(node_has_global_target_hooks)
        }
        FragmentNode::SlotElement(node) => {
            node.fragment.nodes.iter().any(node_has_global_target_hooks)
        }
        FragmentNode::Component(node) => {
            node.fragment.nodes.iter().any(node_has_global_target_hooks)
        }
        FragmentNode::SvelteComponent(node) => {
            node.fragment.nodes.iter().any(node_has_global_target_hooks)
        }
        FragmentNode::SvelteElement(node) => {
            node.fragment.nodes.iter().any(node_has_global_target_hooks)
        }
        FragmentNode::SvelteSelf(node) => {
            node.fragment.nodes.iter().any(node_has_global_target_hooks)
        }
        FragmentNode::SvelteFragment(node) => {
            node.fragment.nodes.iter().any(node_has_global_target_hooks)
        }
        FragmentNode::SvelteHead(_) => false,
        FragmentNode::SvelteBoundary(node) => {
            node.fragment.nodes.iter().any(node_has_global_target_hooks)
        }
        FragmentNode::IfBlock(node) => {
            node.consequent
                .nodes
                .iter()
                .any(node_has_global_target_hooks)
                || node
                    .alternate
                    .as_ref()
                    .is_some_and(|alt| alt.nodes.iter().any(node_has_global_target_hooks))
        }
        FragmentNode::EachBlock(node) => {
            node.body.nodes.iter().any(node_has_global_target_hooks)
                || node
                    .fallback
                    .as_ref()
                    .is_some_and(|alt| alt.nodes.iter().any(node_has_global_target_hooks))
        }
        FragmentNode::AwaitBlock(node) => {
            node.pending
                .as_ref()
                .is_some_and(|alt| alt.nodes.iter().any(node_has_global_target_hooks))
                || node
                    .then
                    .as_ref()
                    .is_some_and(|alt| alt.nodes.iter().any(node_has_global_target_hooks))
                || node
                    .catch
                    .as_ref()
                    .is_some_and(|alt| alt.nodes.iter().any(node_has_global_target_hooks))
        }
        FragmentNode::KeyBlock(node) => {
            node.fragment.nodes.iter().any(node_has_global_target_hooks)
        }
        FragmentNode::SnippetBlock(node) => {
            node.body.nodes.iter().any(node_has_global_target_hooks)
        }
        FragmentNode::Text(_)
        | FragmentNode::ExpressionTag(_)
        | FragmentNode::HtmlTag(_)
        | FragmentNode::ConstTag(_)
        | FragmentNode::DebugTag(_)
        | FragmentNode::RenderTag(_)
        | FragmentNode::AttachTag(_)
        | FragmentNode::Comment(_)
        | FragmentNode::SvelteOptionsRaw(_) => false,
    }
}

fn attributes_have_runtime_target_hooks(attributes: &[AttributeNode<'_>]) -> bool {
    attributes.iter().any(|attribute| {
        matches!(
            attribute,
            AttributeNode::OnDirective(_) | AttributeNode::BindDirective(_)
        )
    })
}

fn push_runtime_helper_import<'a>(
    ast: AstBuilder<'a>,
    body: &mut oxc_allocator::Vec<'a, Statement<'a>>,
    target: TransformTarget,
) {
    let runtime_import_source = match target {
        TransformTarget::Server => LUX_RUNTIME_SERVER_IMPORT_SOURCE,
        TransformTarget::Client => LUX_RUNTIME_CLIENT_IMPORT_SOURCE,
    };
    let parser_allocator = Allocator::default();
    let parsed = Parser::new(&parser_allocator, runtime_import_source, SourceType::mjs()).parse();
    debug_assert!(
        parsed.errors.is_empty(),
        "runtime helper import parse failed: {:?}",
        parsed.errors
    );
    if let Some(statement) = parsed.program.body.first() {
        body.push(statement.clone_in(ast.allocator));
    }
}

fn push_import_declarations<'a>(
    ast: AstBuilder<'a>,
    body: &mut oxc_allocator::Vec<'a, Statement<'a>>,
    root: &Root<'_>,
    analysis: &AnalysisTables,
) {
    let module_import_spans = collect_import_spans(analysis, ScriptTarget::Module);
    let instance_import_spans = collect_import_spans(analysis, ScriptTarget::Instance);

    if let Some(instance_script) = &root.instance {
        push_script_import_declarations(
            ast,
            body,
            &instance_script.content.body,
            &instance_import_spans,
        );
    }
    if let Some(module_script) = &root.module {
        push_script_import_declarations(
            ast,
            body,
            &module_script.content.body,
            &module_import_spans,
        );
    }
}

fn collect_import_spans(analysis: &AnalysisTables, target: ScriptTarget) -> BTreeSet<(u32, u32)> {
    analysis
        .script_imports
        .iter()
        .filter(|item| item.target == target)
        .map(|item| (item.span.start, item.span.end))
        .collect()
}

fn push_script_import_declarations<'a>(
    ast: AstBuilder<'a>,
    body: &mut oxc_allocator::Vec<'a, Statement<'a>>,
    statements: &[Statement<'_>],
    import_spans: &BTreeSet<(u32, u32)>,
) {
    for statement in statements {
        let Statement::ImportDeclaration(declaration) = statement else {
            continue;
        };

        if import_spans.contains(&(declaration.span.start, declaration.span.end)) {
            if let Some(import_statement) = sanitize_import_statement(ast, declaration) {
                body.push(import_statement);
            }
        }
    }
}

fn sanitize_import_statement<'a>(
    ast: AstBuilder<'a>,
    declaration: &oxc_allocator::Box<'_, ImportDeclaration<'_>>,
) -> Option<Statement<'a>> {
    if declaration.import_kind.is_type() {
        return None;
    }

    let mut cloned = declaration.clone_in(ast.allocator);
    if let Some(specifiers) = &declaration.specifiers {
        if specifiers.is_empty() {
            // Normalize `import {} from "x"` to side-effect form `import "x"` for JS output parity.
            cloned.specifiers = None;
        } else {
            let mut kept = ast.vec_with_capacity(specifiers.len());
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        if specifier.import_kind.is_type() {
                            continue;
                        }
                        kept.push(ImportDeclarationSpecifier::ImportSpecifier(
                            specifier.clone_in(ast.allocator),
                        ));
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        kept.push(ImportDeclarationSpecifier::ImportDefaultSpecifier(
                            specifier.clone_in(ast.allocator),
                        ));
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        kept.push(ImportDeclarationSpecifier::ImportNamespaceSpecifier(
                            specifier.clone_in(ast.allocator),
                        ));
                    }
                }
            }
            if kept.is_empty() {
                return None;
            }
            cloned.specifiers = Some(kept);
        }
    }

    Some(Statement::ImportDeclaration(cloned))
}

fn collect_instance_import_names(analysis: &AnalysisTables) -> Vec<String> {
    let mut names = BTreeSet::new();
    // Runtime helper aliases are local bindings in generated module scope.
    names.insert(LUX_STRINGIFY.to_string());
    names.insert(LUX_ESCAPE.to_string());
    names.insert(LUX_ESCAPE_ATTR.to_string());
    names.insert(LUX_ATTR.to_string());
    names.insert(LUX_CLASS_ATTR.to_string());
    names.insert(LUX_STYLE_ATTR.to_string());
    names.insert(LUX_ATTRIBUTES.to_string());
    names.insert(LUX_IS_BOOLEAN_ATTR.to_string());
    names.insert(LUX_PROPS_ID.to_string());
    names.insert(LUX_MOUNT_HTML.to_string());
    names.insert(LUX_CLEANUP_MOUNT.to_string());
    names.insert(LUX_BEGIN_RENDER.to_string());
    names.insert(LUX_END_RENDER.to_string());
    names.insert(LUX_EVENT_ATTR.to_string());
    names.insert(LUX_EVENT_TARGET_ATTR.to_string());
    names.insert(LUX_MOUNT_EVENTS.to_string());
    names.insert(LUX_ONCE.to_string());
    names.insert(LUX_BIND_ATTR.to_string());
    names.insert(LUX_BIND_TARGET_ATTR.to_string());
    names.insert(LUX_MOUNT_BINDINGS.to_string());
    names.insert(LUX_USE_ATTR.to_string());
    names.insert(LUX_MOUNT_ACTIONS.to_string());
    names.insert(LUX_TRANSITION_ATTR.to_string());
    names.insert(LUX_MOUNT_TRANSITIONS.to_string());
    names.insert(LUX_ANIMATE_ATTR.to_string());
    names.insert(LUX_MOUNT_ANIMATIONS.to_string());
    for import in &analysis.script_imports {
        if import.target != ScriptTarget::Instance {
            continue;
        }
        for name in &import.local_names {
            names.insert(name.clone());
        }
    }
    names.into_iter().collect()
}
