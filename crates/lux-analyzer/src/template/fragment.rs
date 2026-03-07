use lux_ast::analysis::{AnalysisDiagnosticCode, AnalysisSeverity};
use lux_ast::common::Span;
use lux_ast::template::attribute::AttributeNode;
use lux_ast::template::root::{Fragment, FragmentNode};
use oxc_ast::ast::Expression;

use super::context::TemplateAnalyzerContext;
use super::diagnostics::{self, BindDirectiveTarget};
use super::node;
use super::node::element::ElementContainerKind;
use super::reference;

pub(super) fn analyze_fragment(fragment: &Fragment<'_>, context: &mut TemplateAnalyzerContext<'_>) {
    maybe_report_slot_render_conflict(fragment, context);

    for node in &fragment.nodes {
        analyze_node(node, context);
    }
}

fn analyze_node(node: &FragmentNode<'_>, context: &mut TemplateAnalyzerContext<'_>) {
    match node {
        FragmentNode::Text(_)
        | FragmentNode::ConstTag(_)
        | FragmentNode::DebugTag(_)
        | FragmentNode::Comment(_) => {}

        FragmentNode::ExpressionTag(tag) => {
            reference::analyze_expression(&tag.expression, context);
        }
        FragmentNode::HtmlTag(tag) => {
            reference::analyze_expression(&tag.expression, context);
        }
        FragmentNode::RenderTag(tag) => {
            diagnostics::validate_render_tag(tag, context);
            reference::analyze_expression(&tag.expression, context);
        }
        FragmentNode::AttachTag(tag) => {
            reference::analyze_expression(&tag.expression, context);
        }

        FragmentNode::IfBlock(block) => {
            diagnostics::warn_if_block_empty(&block.consequent, context);
            if let Some(alternate) = &block.alternate {
                diagnostics::warn_if_block_empty(alternate, context);
            }

            reference::analyze_expression(&block.test, context);
            context.with_nested_region(|context| {
                analyze_fragment(&block.consequent, context);
            });
            if let Some(alternate) = &block.alternate {
                context.with_nested_region(|context| {
                    analyze_fragment(alternate, context);
                });
            }
        }
        FragmentNode::EachBlock(block) => node::each_block::analyze(block, context),
        FragmentNode::AwaitBlock(block) => node::await_block::analyze(block, context),
        FragmentNode::KeyBlock(block) => {
            diagnostics::warn_if_block_empty(&block.fragment, context);
            reference::analyze_expression(&block.expression, context);
            context.with_nested_region(|context| {
                analyze_fragment(&block.fragment, context);
            });
        }
        FragmentNode::SnippetBlock(block) => node::snippet_block::analyze(block, context),

        FragmentNode::RegularElement(element) => {
            node::element::analyze(
                ElementContainerKind::Regular,
                BindDirectiveTarget::Regular(element.name),
                true, // `let:` allowed on regular elements
                element.span,
                &element.attributes,
                &element.fragment,
                context,
            );
        }
        FragmentNode::Component(component) => {
            node::element::analyze(
                ElementContainerKind::Component,
                BindDirectiveTarget::Other,
                true, // `let:` allowed on components
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteElement(element) => {
            reference::analyze_expression(&element.tag, context);
            node::element::analyze(
                ElementContainerKind::Other,
                BindDirectiveTarget::SvelteElement,
                true, // `let:` allowed on <svelte:element>
                element.span,
                &element.attributes,
                &element.fragment,
                context,
            );
        }
        FragmentNode::SvelteComponent(component) => {
            reference::analyze_expression(&component.expression, context);
            node::element::analyze(
                ElementContainerKind::SvelteComponent,
                BindDirectiveTarget::Other,
                true, // `let:` allowed on <svelte:component>
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteSelf(component) => {
            node::element::analyze(
                ElementContainerKind::SvelteSelf,
                BindDirectiveTarget::Other,
                true, // `let:` allowed on <svelte:self>
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteFragment(component) => {
            node::element::analyze(
                ElementContainerKind::Other,
                BindDirectiveTarget::Other,
                true, // `let:` allowed on <svelte:fragment>
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteHead(component) => {
            for attribute in &component.attributes {
                context.add_diagnostic(
                    AnalysisSeverity::Error,
                    AnalysisDiagnosticCode::SvelteHeadIllegalAttribute,
                    "`<svelte:head>` cannot have attributes nor directives",
                    attribute_node_span(attribute),
                );
            }
            maybe_report_meta_invalid_placement(context, "svelte:head", component.span);
            let head_seen = context.mark_svelte_head_seen();
            maybe_report_meta_duplicate(context, "svelte:head", component.span, head_seen);
            node::element::analyze(
                ElementContainerKind::Other,
                BindDirectiveTarget::Other,
                false, // `let:` NOT allowed on <svelte:head>
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteBody(component) => {
            maybe_report_meta_invalid_placement(context, "svelte:body", component.span);
            let body_seen = context.mark_svelte_body_seen();
            maybe_report_meta_duplicate(context, "svelte:body", component.span, body_seen);
            maybe_report_meta_invalid_content(
                context,
                "svelte:body",
                component.fragment.nodes.is_empty(),
                component.span,
            );
            node::element::analyze(
                ElementContainerKind::Other,
                BindDirectiveTarget::SvelteBody,
                false, // `let:` NOT allowed on <svelte:body>
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteWindow(component) => {
            maybe_report_meta_invalid_placement(context, "svelte:window", component.span);
            let window_seen = context.mark_svelte_window_seen();
            maybe_report_meta_duplicate(context, "svelte:window", component.span, window_seen);
            maybe_report_meta_invalid_content(
                context,
                "svelte:window",
                component.fragment.nodes.is_empty(),
                component.span,
            );
            node::element::analyze(
                ElementContainerKind::Other,
                BindDirectiveTarget::SvelteWindow,
                false, // `let:` NOT allowed on <svelte:window>
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteDocument(component) => {
            maybe_report_meta_invalid_placement(context, "svelte:document", component.span);
            let document_seen = context.mark_svelte_document_seen();
            maybe_report_meta_duplicate(context, "svelte:document", component.span, document_seen);
            maybe_report_meta_invalid_content(
                context,
                "svelte:document",
                component.fragment.nodes.is_empty(),
                component.span,
            );
            node::element::analyze(
                ElementContainerKind::Other,
                BindDirectiveTarget::SvelteDocument,
                false, // `let:` NOT allowed on <svelte:document>
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteBoundary(component) => {
            node::element::analyze(
                ElementContainerKind::Other,
                BindDirectiveTarget::Other,
                false, // `let:` NOT allowed on <svelte:boundary>
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SlotElement(element) => {
            node::element::analyze(
                ElementContainerKind::Other,
                BindDirectiveTarget::Other,
                true, // `let:` allowed on <slot>
                element.span,
                &element.attributes,
                &element.fragment,
                context,
            );
        }
        FragmentNode::TitleElement(element) => {
            node::element::analyze(
                ElementContainerKind::Other,
                BindDirectiveTarget::Other,
                false, // `let:` NOT allowed on <title>
                element.span,
                &element.attributes,
                &element.fragment,
                context,
            );
        }
        FragmentNode::SvelteOptionsRaw(element) => {
            maybe_report_meta_invalid_placement(context, "svelte:options", element.span);
            let options_seen = context.mark_svelte_options_seen();
            maybe_report_meta_duplicate(context, "svelte:options", element.span, options_seen);
            maybe_report_meta_invalid_content(
                context,
                "svelte:options",
                element.fragment.nodes.is_empty(),
                element.span,
            );
            node::element::analyze(
                ElementContainerKind::Other,
                BindDirectiveTarget::Other,
                false, // `let:` NOT allowed on <svelte:options>
                element.span,
                &element.attributes,
                &element.fragment,
                context,
            );
        }
    }
}

fn maybe_report_meta_invalid_placement(
    context: &mut TemplateAnalyzerContext<'_>,
    name: &str,
    span: lux_ast::common::Span,
) {
    if context.is_inside_element_or_block() {
        context.add_diagnostic(
            AnalysisSeverity::Error,
            AnalysisDiagnosticCode::SvelteMetaInvalidPlacement,
            format!("`<{name}>` tags cannot be inside elements or blocks"),
            span,
        );
    }
}

fn maybe_report_meta_invalid_content(
    context: &mut TemplateAnalyzerContext<'_>,
    name: &str,
    is_empty: bool,
    span: lux_ast::common::Span,
) {
    if !is_empty {
        context.add_diagnostic(
            AnalysisSeverity::Error,
            AnalysisDiagnosticCode::SvelteMetaInvalidContent,
            format!("<{name}> cannot have children"),
            span,
        );
    }
}

fn maybe_report_meta_duplicate(
    context: &mut TemplateAnalyzerContext<'_>,
    name: &str,
    span: lux_ast::common::Span,
    already_seen: bool,
) {
    if already_seen {
        context.add_diagnostic(
            AnalysisSeverity::Error,
            AnalysisDiagnosticCode::SvelteMetaDuplicate,
            format!("A component can only have one `<{name}>` element"),
            span,
        );
    }
}

fn maybe_report_slot_render_conflict(
    fragment: &Fragment<'_>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    let has_slot = fragment
        .nodes
        .iter()
        .any(|node| matches!(node, FragmentNode::SlotElement(_)));
    if !has_slot {
        return;
    }

    let render_children_span = fragment.nodes.iter().find_map(|node| match node {
        FragmentNode::RenderTag(tag) if is_children_render_expression(&tag.expression) => {
            Some(tag.span)
        }
        _ => None,
    });

    let Some(span) = render_children_span else {
        return;
    };

    context.add_diagnostic(
        AnalysisSeverity::Error,
        AnalysisDiagnosticCode::SnippetChildrenConflict,
        "Cannot use `<slot>` and `{@render children(...)}` in the same component",
        span,
    );
}

fn is_children_render_expression(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::CallExpression(call) => is_children_identifier_expression(&call.callee),
        Expression::ParenthesizedExpression(expression) => {
            is_children_render_expression(&expression.expression)
        }
        _ => false,
    }
}

fn is_children_identifier_expression(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(identifier) => identifier.name == "children",
        Expression::ParenthesizedExpression(expression) => {
            is_children_identifier_expression(&expression.expression)
        }
        _ => false,
    }
}

fn attribute_node_span(attribute: &AttributeNode<'_>) -> Span {
    match attribute {
        AttributeNode::Attribute(attribute) => attribute.span,
        AttributeNode::SpreadAttribute(attribute) => attribute.span,
        AttributeNode::BindDirective(attribute) => attribute.span,
        AttributeNode::ClassDirective(attribute) => attribute.span,
        AttributeNode::StyleDirective(attribute) => attribute.span,
        AttributeNode::OnDirective(attribute) => attribute.span,
        AttributeNode::TransitionDirective(attribute) => attribute.span,
        AttributeNode::AnimateDirective(attribute) => attribute.span,
        AttributeNode::UseDirective(attribute) => attribute.span,
        AttributeNode::LetDirective(attribute) => attribute.span,
        AttributeNode::AttachTag(attribute) => attribute.span,
    }
}
