mod attributes;
mod blocks;
mod elements;
mod expr;
mod scope;

use lux_ast::template::root::{Fragment, FragmentNode};
use oxc_allocator::CloneIn;
use oxc_ast::{AstBuilder, ast::Expression};

use self::blocks::{
    render_await_block_expression, render_each_block_expression, render_if_block_expression,
    render_snippet_block_declaration,
};
use self::elements::{
    render_component_expression, render_regular_element_expression, render_svelte_component_expression,
    render_svelte_element_expression,
};
use self::expr::{
    concat_expr, dynamic_marker_expr, escape_html_expression, string_expr, stringify_expression,
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
    let mut out = string_expr(ast, "");
    let mut current_scope = scope.clone();

    for node in &fragment.nodes {
        match node {
            FragmentNode::SnippetBlock(block) => {
                out = concat_expr(
                    ast,
                    out,
                    render_snippet_block_declaration(ast, block, &current_scope),
                );
                current_scope = current_scope.with_name(block.expression.name.as_str());
            }
            _ => {
                out = concat_expr(ast, out, render_node_expression(ast, node, &current_scope));
            }
        }
    }

    out
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

        FragmentNode::ConstTag(_) => dynamic_marker_expr(ast, "const-tag"),
        FragmentNode::DebugTag(_) => dynamic_marker_expr(ast, "debug-tag"),
        FragmentNode::RenderTag(tag) => stringify_expression(
            ast,
            resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
        ),
        FragmentNode::AttachTag(_) => dynamic_marker_expr(ast, "attach-tag"),
        FragmentNode::SnippetBlock(_) => string_expr(ast, ""),
        FragmentNode::Component(component) => render_component_expression(ast, component, scope),
        FragmentNode::SvelteComponent(component) => {
            render_svelte_component_expression(ast, component, scope)
        }
        FragmentNode::SvelteElement(element) => render_svelte_element_expression(ast, element, scope),
        FragmentNode::SvelteSelf(_) => dynamic_marker_expr(ast, "svelte-self"),
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
        FragmentNode::SvelteOptionsRaw(_) => dynamic_marker_expr(ast, "svelte-options"),
    }
}
