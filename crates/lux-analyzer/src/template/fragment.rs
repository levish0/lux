use lux_ast::analysis::{AnalysisDiagnosticCode, AnalysisSeverity};
use lux_ast::template::root::{Fragment, FragmentNode};

use super::context::TemplateAnalyzerContext;
use super::diagnostics::{self, BindDirectiveTarget};
use super::node;
use super::node::element::ElementContainerKind;
use super::reference;

pub(super) fn analyze_fragment(fragment: &Fragment<'_>, context: &mut TemplateAnalyzerContext<'_>) {
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
