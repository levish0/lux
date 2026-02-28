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
            analyze_fragment(&block.consequent, context);
            if let Some(alternate) = &block.alternate {
                analyze_fragment(alternate, context);
            }
        }
        FragmentNode::EachBlock(block) => node::each_block::analyze(block, context),
        FragmentNode::AwaitBlock(block) => node::await_block::analyze(block, context),
        FragmentNode::KeyBlock(block) => {
            diagnostics::warn_if_block_empty(&block.fragment, context);
            reference::analyze_expression(&block.expression, context);
            analyze_fragment(&block.fragment, context);
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
