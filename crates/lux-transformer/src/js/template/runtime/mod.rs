mod attributes;
mod blocks;
mod elements;
mod expr;
mod scope;

use lux_ast::template::root::{Fragment, FragmentNode};
use oxc_allocator::CloneIn;
use oxc_ast::{AstBuilder, ast::Expression};
use oxc_ast::ast::PropertyKind;
use oxc_span::SPAN;

use self::blocks::{
    render_await_block_expression, render_const_tag_declaration_statement,
    render_each_block_expression, render_if_block_expression, render_snippet_block_declaration,
};
use self::elements::{
    render_component_expression, render_regular_element_expression, render_svelte_component_expression,
    render_svelte_element_expression, render_svelte_self_expression,
};
use self::expr::{
    call_iife, call_static_method, escape_html_expression, string_expr, stringify_expression,
};
use self::scope::{RuntimeScope, resolve_expression};
use super::marker::sanitize_comment;

pub(super) fn build_render_expression<'a>(
    ast: AstBuilder<'a>,
    fragment: &Fragment<'_>,
) -> Expression<'a> {
    render_fragment_expression(ast, fragment, &RuntimeScope::default())
}

fn render_fragment_expression<'a>(
    ast: AstBuilder<'a>,
    fragment: &Fragment<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let mut statements = ast.vec();
    let chunks_ident = ast.expression_identifier(SPAN, ast.ident("__lux_chunks"));
    let chunks_declarator = ast.variable_declarator(
        SPAN,
        oxc_ast::ast::VariableDeclarationKind::Let,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident("__lux_chunks")),
        oxc_ast::NONE,
        Some(ast.expression_array(SPAN, ast.vec())),
        false,
    );
    statements.push(
        ast.declaration_variable(
            SPAN,
            oxc_ast::ast::VariableDeclarationKind::Let,
            ast.vec1(chunks_declarator),
            false,
        )
        .into(),
    );

    let mut current_scope = scope.clone();

    for node in &fragment.nodes {
        match node {
            FragmentNode::ConstTag(tag) => {
                statements.push(render_const_tag_declaration_statement(
                    ast,
                    tag,
                    &current_scope,
                ));
                current_scope = current_scope.with_binding_pattern(&tag.declaration.id);
            }
            FragmentNode::SnippetBlock(block) => {
                let rendered = render_snippet_block_declaration(ast, block, &current_scope);
                statements.push(
                    ast.statement_expression(
                        SPAN,
                        call_static_method(
                            ast,
                            chunks_ident.clone_in(ast.allocator),
                            "push",
                            ast.vec1(rendered.into()),
                        ),
                    ),
                );
                current_scope = current_scope.with_name(block.expression.name.as_str());
            }
            _ => {
                let rendered = render_node_expression(ast, node, &current_scope);
                statements.push(
                    ast.statement_expression(
                        SPAN,
                        call_static_method(
                            ast,
                            chunks_ident.clone_in(ast.allocator),
                            "push",
                            ast.vec1(rendered.into()),
                        ),
                    ),
                );
            }
        }
    }

    let joined = call_static_method(
        ast,
        chunks_ident,
        "join",
        ast.vec1(string_expr(ast, "").into()),
    );
    statements.push(ast.statement_return(SPAN, Some(joined)));
    call_iife(ast, statements)
}

fn render_node_expression<'a>(
    ast: AstBuilder<'a>,
    node: &FragmentNode<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    match node {
        FragmentNode::Text(text) => string_expr(ast, text.raw),
        FragmentNode::Comment(comment) => {
            let value = format!("<!--{}-->", sanitize_comment(comment.data));
            string_expr(ast, &value)
        }

        FragmentNode::RegularElement(element) => render_regular_element_expression(
            ast,
            element.name,
            &element.attributes,
            &element.fragment,
            scope,
        ),
        FragmentNode::TitleElement(element) => render_regular_element_expression(
            ast,
            element.name,
            &element.attributes,
            &element.fragment,
            scope,
        ),
        FragmentNode::SlotElement(element) => render_regular_element_expression(
            ast,
            element.name,
            &element.attributes,
            &element.fragment,
            scope,
        ),

        FragmentNode::ExpressionTag(tag) => escape_html_expression(
            ast,
            stringify_expression(
                ast,
                resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
            ),
        ),
        FragmentNode::HtmlTag(tag) => stringify_expression(
            ast,
            resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
        ),
        FragmentNode::IfBlock(block) => render_if_block_expression(ast, block, scope),
        FragmentNode::EachBlock(block) => render_each_block_expression(ast, block, scope),
        FragmentNode::AwaitBlock(block) => render_await_block_expression(ast, block, scope),
        FragmentNode::KeyBlock(block) => render_fragment_expression(ast, &block.fragment, scope),

        FragmentNode::ConstTag(_) => string_expr(ast, ""),
        FragmentNode::DebugTag(tag) => render_debug_tag_expression(ast, tag, scope),
        FragmentNode::RenderTag(tag) => stringify_expression(
            ast,
            resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
        ),
        FragmentNode::AttachTag(_) => string_expr(ast, ""),
        FragmentNode::SnippetBlock(_) => string_expr(ast, ""),
        FragmentNode::Component(component) => render_component_expression(ast, component, scope),
        FragmentNode::SvelteComponent(component) => {
            render_svelte_component_expression(ast, component, scope)
        }
        FragmentNode::SvelteElement(element) => render_svelte_element_expression(ast, element, scope),
        FragmentNode::SvelteSelf(component) => render_svelte_self_expression(ast, component, scope),
        FragmentNode::SvelteFragment(element) => render_fragment_expression(ast, &element.fragment, scope),
        FragmentNode::SvelteHead(element) => render_fragment_expression(ast, &element.fragment, scope),
        FragmentNode::SvelteBody(element) => render_fragment_expression(ast, &element.fragment, scope),
        FragmentNode::SvelteWindow(element) => render_fragment_expression(ast, &element.fragment, scope),
        FragmentNode::SvelteDocument(element) => {
            render_fragment_expression(ast, &element.fragment, scope)
        }
        FragmentNode::SvelteBoundary(element) => {
            render_fragment_expression(ast, &element.fragment, scope)
        }
        FragmentNode::SvelteOptionsRaw(_) => string_expr(ast, ""),
    }
}

fn render_debug_tag_expression<'a>(
    ast: AstBuilder<'a>,
    tag: &lux_ast::template::tag::DebugTag<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let mut properties = ast.vec_with_capacity(tag.identifiers.len());
    for identifier in &tag.identifiers {
        let name = identifier.name.as_str();
        let value = resolve_expression(
            ast,
            ast.expression_identifier(SPAN, ast.ident(name)),
            scope,
        );
        properties.push(ast.object_property_kind_object_property(
            SPAN,
            PropertyKind::Init,
            ast.property_key_static_identifier(SPAN, ast.ident(name)),
            value,
            false,
            false,
            false,
        ));
    }

    let object = ast.expression_object(SPAN, properties);
    let console_log = ast.member_expression_static(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("console")),
        ast.identifier_name(SPAN, ast.ident("log")),
        false,
    );
    let call = ast.expression_call(
        SPAN,
        console_log.into(),
        oxc_ast::NONE,
        ast.vec1(object.into()),
        false,
    );

    ast.expression_sequence(SPAN, ast.vec_from_array([call, string_expr(ast, "")]))
}
