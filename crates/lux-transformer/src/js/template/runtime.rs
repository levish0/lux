use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::block::{EachBlock, IfBlock};
use lux_ast::template::root::{Fragment, FragmentNode};
use lux_ast::template::tag::TextOrExpressionTag;
use lux_utils::elements::is_void;
use oxc_allocator::CloneIn;
use oxc_ast::{
    AstBuilder, NONE,
    ast::{
        BinaryOperator, BindingPattern, Expression, FormalParameterKind, FunctionType,
        IdentifierReference, LogicalOperator,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::SPAN;
use rustc_hash::FxHashSet;

use super::marker::sanitize_comment;

pub(super) fn build_render_expression<'a>(
    ast: AstBuilder<'a>,
    fragment: &Fragment<'_>,
) -> Expression<'a> {
    render_fragment_expression(ast, fragment, &RuntimeScope::default())
}

#[derive(Default, Clone)]
struct RuntimeScope {
    local_bindings: FxHashSet<String>,
}

impl RuntimeScope {
    fn contains(&self, name: &str) -> bool {
        self.local_bindings.contains(name)
    }

    fn with_binding_pattern(&self, pattern: &BindingPattern<'_>) -> Self {
        let mut next = self.clone();
        collect_binding_pattern_names(pattern, &mut next.local_bindings);
        next
    }

    fn with_name(&self, name: &str) -> Self {
        let mut next = self.clone();
        next.local_bindings.insert(name.to_string());
        next
    }
}

fn render_fragment_expression<'a>(
    ast: AstBuilder<'a>,
    fragment: &Fragment<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let mut out = string_expr(ast, "");
    for node in &fragment.nodes {
        out = concat_expr(ast, out, render_node_expression(ast, node, scope));
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

        FragmentNode::ExpressionTag(tag) => stringify_expression(
            ast,
            resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
        ),
        FragmentNode::HtmlTag(tag) => stringify_expression(
            ast,
            resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
        ),
        FragmentNode::IfBlock(block) => render_if_block_expression(ast, block, scope),
        FragmentNode::EachBlock(block) => render_each_block_expression(ast, block, scope),
        FragmentNode::KeyBlock(block) => render_fragment_expression(ast, &block.fragment, scope),

        FragmentNode::ConstTag(_) => dynamic_marker_expr(ast, "const-tag"),
        FragmentNode::DebugTag(_) => dynamic_marker_expr(ast, "debug-tag"),
        FragmentNode::RenderTag(tag) => stringify_expression(
            ast,
            resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
        ),
        FragmentNode::AttachTag(_) => dynamic_marker_expr(ast, "attach-tag"),
        FragmentNode::AwaitBlock(_) => dynamic_marker_expr(ast, "await-block"),
        FragmentNode::SnippetBlock(_) => dynamic_marker_expr(ast, "snippet-block"),
        FragmentNode::Component(_) => dynamic_marker_expr(ast, "component"),
        FragmentNode::SvelteComponent(_) => dynamic_marker_expr(ast, "svelte-component"),
        FragmentNode::SvelteElement(_) => dynamic_marker_expr(ast, "svelte-element"),
        FragmentNode::SvelteSelf(_) => dynamic_marker_expr(ast, "svelte-self"),
        FragmentNode::SvelteFragment(_) => dynamic_marker_expr(ast, "svelte-fragment"),
        FragmentNode::SvelteHead(_) => dynamic_marker_expr(ast, "svelte-head"),
        FragmentNode::SvelteBody(_) => dynamic_marker_expr(ast, "svelte-body"),
        FragmentNode::SvelteWindow(_) => dynamic_marker_expr(ast, "svelte-window"),
        FragmentNode::SvelteDocument(_) => dynamic_marker_expr(ast, "svelte-document"),
        FragmentNode::SvelteBoundary(_) => dynamic_marker_expr(ast, "svelte-boundary"),
        FragmentNode::SvelteOptionsRaw(_) => dynamic_marker_expr(ast, "svelte-options"),
    }
}

fn render_regular_element_expression<'a>(
    ast: AstBuilder<'a>,
    name: &str,
    attributes: &[AttributeNode<'_>],
    children: &Fragment<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let mut out = string_expr(ast, &format!("<{name}"));

    for attribute in attributes {
        out = concat_expr(ast, out, render_attribute_expression(ast, attribute, scope));
    }

    out = concat_expr(ast, out, string_expr(ast, ">"));
    if !is_void(name) {
        out = concat_expr(ast, out, render_fragment_expression(ast, children, scope));
        out = concat_expr(ast, out, string_expr(ast, &format!("</{name}>")));
    }

    out
}

fn render_attribute_expression<'a>(
    ast: AstBuilder<'a>,
    attribute: &AttributeNode<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let AttributeNode::Attribute(attribute) = attribute else {
        return string_expr(ast, "");
    };

    match &attribute.value {
        AttributeValue::True => string_expr(ast, &format!(" {}", attribute.name)),
        AttributeValue::ExpressionTag(tag) => {
            let mut out = string_expr(ast, &format!(" {}=\"", attribute.name));
            out = concat_expr(
                ast,
                out,
                stringify_expression(
                    ast,
                    resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
                ),
            );
            concat_expr(ast, out, string_expr(ast, "\""))
        }
        AttributeValue::Sequence(chunks) => {
            let mut value_expr = string_expr(ast, "");
            for chunk in chunks {
                let chunk_expr = match chunk {
                    TextOrExpressionTag::Text(text) => string_expr(ast, text.raw),
                    TextOrExpressionTag::ExpressionTag(tag) => stringify_expression(
                        ast,
                        resolve_expression(ast, tag.expression.clone_in(ast.allocator), scope),
                    ),
                };
                value_expr = concat_expr(ast, value_expr, chunk_expr);
            }

            let mut out = string_expr(ast, &format!(" {}=\"", attribute.name));
            out = concat_expr(ast, out, value_expr);
            concat_expr(ast, out, string_expr(ast, "\""))
        }
    }
}

fn render_if_block_expression<'a>(
    ast: AstBuilder<'a>,
    block: &IfBlock<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let test = resolve_expression(ast, block.test.clone_in(ast.allocator), scope);
    let consequent = render_fragment_expression(ast, &block.consequent, scope);
    let alternate = block.alternate.as_ref().map_or_else(
        || string_expr(ast, ""),
        |alternate| render_fragment_expression(ast, alternate, scope),
    );

    ast.expression_conditional(SPAN, test, consequent, alternate)
}

fn render_each_block_expression<'a>(
    ast: AstBuilder<'a>,
    block: &EachBlock<'_>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let source = resolve_expression(ast, block.expression.clone_in(ast.allocator), scope);
    let iterable = ast.expression_logical(
        SPAN,
        source,
        LogicalOperator::Coalesce,
        ast.expression_array(SPAN, ast.vec()),
    );
    let from_call = call_static_method(
        ast,
        ast.expression_identifier(SPAN, ast.ident("Array")),
        "from",
        ast.vec1(iterable.into()),
    );

    let mut params_items = ast.vec_with_capacity(if block.index.is_some() { 2 } else { 1 });
    let context_pattern = block.context.as_ref().map_or_else(
        || ast.binding_pattern_binding_identifier(SPAN, ast.ident("__item")),
        |pattern| pattern.clone_in(ast.allocator),
    );
    params_items.push(ast.formal_parameter(
        SPAN,
        ast.vec(),
        context_pattern,
        NONE,
        NONE,
        false,
        None,
        false,
        false,
    ));
    if let Some(index) = block.index {
        params_items.push(ast.formal_parameter(
            SPAN,
            ast.vec(),
            ast.binding_pattern_binding_identifier(SPAN, ast.ident(index)),
            NONE,
            NONE,
            false,
            None,
            false,
            false,
        ));
    }

    let params = ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        params_items,
        NONE,
    );
    let mut body_scope = scope.clone();
    if let Some(context) = &block.context {
        body_scope = body_scope.with_binding_pattern(context);
    }
    if let Some(index) = block.index {
        body_scope = body_scope.with_name(index);
    }

    let body_expr = render_fragment_expression(ast, &block.body, &body_scope);
    let body = ast.alloc_function_body(
        SPAN,
        ast.vec(),
        ast.vec1(ast.statement_return(SPAN, Some(body_expr))),
    );
    let callback = ast.expression_function(
        SPAN,
        FunctionType::FunctionExpression,
        None,
        false,
        false,
        false,
        NONE,
        NONE,
        params,
        NONE,
        Some(body),
    );

    let mapped = call_static_method(
        ast,
        from_call.clone_in(ast.allocator),
        "map",
        ast.vec1(callback.into()),
    );
    let joined = call_static_method(ast, mapped, "join", ast.vec1(string_expr(ast, "").into()));

    if let Some(fallback) = &block.fallback {
        let fallback_expr = render_fragment_expression(ast, fallback, scope);
        let len_expr = ast.member_expression_static(
            SPAN,
            from_call,
            ast.identifier_name(SPAN, ast.ident("length")),
            false,
        );
        let has_items = ast.expression_binary(
            SPAN,
            len_expr.into(),
            BinaryOperator::GreaterThan,
            ast.expression_numeric_literal(SPAN, 0.0, None, oxc_ast::ast::NumberBase::Decimal),
        );
        ast.expression_conditional(SPAN, has_items, joined, fallback_expr)
    } else {
        joined
    }
}

fn call_static_method<'a>(
    ast: AstBuilder<'a>,
    object: Expression<'a>,
    method: &str,
    arguments: oxc_allocator::Vec<'a, oxc_ast::ast::Argument<'a>>,
) -> Expression<'a> {
    let callee = ast.member_expression_static(
        SPAN,
        object,
        ast.identifier_name(SPAN, ast.ident(method)),
        false,
    );
    ast.expression_call(SPAN, callee.into(), NONE, arguments, false)
}

fn stringify_expression<'a>(ast: AstBuilder<'a>, expression: Expression<'a>) -> Expression<'a> {
    let value = ast.expression_logical(
        SPAN,
        expression,
        LogicalOperator::Coalesce,
        string_expr(ast, ""),
    );
    ast.expression_call(
        SPAN,
        ast.expression_identifier(SPAN, ast.ident("String")),
        NONE,
        ast.vec1(value.into()),
        false,
    )
}

fn resolve_expression<'a>(
    ast: AstBuilder<'a>,
    expression: Expression<'a>,
    scope: &RuntimeScope,
) -> Expression<'a> {
    let mut collector = IdentifierCollector::default();
    collector.visit_expression(&expression);

    let mut names = collector
        .names
        .into_iter()
        .filter(|name| !scope.contains(name))
        .filter(|name| !is_runtime_global(name))
        .collect::<Vec<_>>();

    if names.is_empty() {
        return expression;
    }

    names.sort_unstable();
    names.dedup();

    let mut properties = ast.vec_with_capacity(names.len());
    for name in names {
        properties.push(ast.binding_property(
            SPAN,
            ast.property_key_static_identifier(SPAN, ast.ident(name)),
            ast.binding_pattern_binding_identifier(SPAN, ast.ident(name)),
            true,
            false,
        ));
    }

    let props_pattern = ast.binding_pattern_object_pattern(SPAN, properties, NONE);
    let params = ast.alloc_formal_parameters(
        SPAN,
        FormalParameterKind::FormalParameter,
        ast.vec1(ast.formal_parameter(
            SPAN,
            ast.vec(),
            props_pattern,
            NONE,
            NONE,
            false,
            None,
            false,
            false,
        )),
        NONE,
    );
    let body = ast.alloc_function_body(
        SPAN,
        ast.vec(),
        ast.vec1(ast.statement_return(SPAN, Some(expression))),
    );
    let resolver = ast.expression_function(
        SPAN,
        FunctionType::FunctionExpression,
        None,
        false,
        false,
        false,
        NONE,
        NONE,
        params,
        NONE,
        Some(body),
    );

    ast.expression_call(
        SPAN,
        resolver,
        NONE,
        ast.vec1(ast.expression_identifier(SPAN, ast.ident("_props")).into()),
        false,
    )
}

#[derive(Default)]
struct IdentifierCollector<'a> {
    names: FxHashSet<&'a str>,
}

impl<'a> Visit<'a> for IdentifierCollector<'a> {
    fn visit_expression(&mut self, it: &Expression<'a>) {
        walk::walk_expression(self, it);
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        self.names.insert(it.name.as_str());
    }
}

fn collect_binding_pattern_names(pattern: &BindingPattern<'_>, names: &mut FxHashSet<String>) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            names.insert(identifier.name.as_str().to_string());
        }
        BindingPattern::ObjectPattern(pattern) => {
            for property in &pattern.properties {
                collect_binding_pattern_names(&property.value, names);
            }
            if let Some(rest) = &pattern.rest {
                collect_binding_pattern_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(pattern) => {
            for element in &pattern.elements {
                if let Some(element) = element {
                    collect_binding_pattern_names(element, names);
                }
            }
            if let Some(rest) = &pattern.rest {
                collect_binding_pattern_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(pattern) => {
            collect_binding_pattern_names(&pattern.left, names);
        }
    }
}

fn is_runtime_global(name: &str) -> bool {
    matches!(
        name,
        "_props"
            | "undefined"
            | "Infinity"
            | "NaN"
            | "Math"
            | "Number"
            | "String"
            | "Boolean"
            | "Object"
            | "Array"
            | "Date"
            | "JSON"
            | "RegExp"
            | "Map"
            | "Set"
            | "WeakMap"
            | "WeakSet"
            | "Promise"
            | "Symbol"
            | "BigInt"
            | "console"
            | "window"
            | "document"
            | "globalThis"
    )
}

fn dynamic_marker_expr<'a>(ast: AstBuilder<'a>, kind: &str) -> Expression<'a> {
    string_expr(ast, &format!("<!--lux:dynamic:{kind}-->"))
}

fn string_expr<'a>(ast: AstBuilder<'a>, value: &str) -> Expression<'a> {
    ast.expression_string_literal(SPAN, ast.atom(value), None)
}

fn concat_expr<'a>(
    ast: AstBuilder<'a>,
    left: Expression<'a>,
    right: Expression<'a>,
) -> Expression<'a> {
    ast.expression_binary(SPAN, left, BinaryOperator::Addition, right)
}
