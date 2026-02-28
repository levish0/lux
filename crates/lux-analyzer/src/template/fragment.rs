use lux_ast::template::root::{Fragment, FragmentNode};

use super::context::TemplateAnalyzerContext;
use super::node;

pub(super) fn analyze_fragment(fragment: &Fragment<'_>, context: &mut TemplateAnalyzerContext<'_>) {
    for node in &fragment.nodes {
        analyze_node(node, context);
    }
}

fn analyze_node(node: &FragmentNode<'_>, context: &mut TemplateAnalyzerContext<'_>) {
    match node {
        FragmentNode::Text(_)
        | FragmentNode::ExpressionTag(_)
        | FragmentNode::HtmlTag(_)
        | FragmentNode::ConstTag(_)
        | FragmentNode::DebugTag(_)
        | FragmentNode::RenderTag(_)
        | FragmentNode::AttachTag(_)
        | FragmentNode::Comment(_) => {}

        FragmentNode::IfBlock(block) => {
            analyze_fragment(&block.consequent, context);
            if let Some(alternate) = &block.alternate {
                analyze_fragment(alternate, context);
            }
        }
        FragmentNode::EachBlock(block) => node::each_block::analyze(block, context),
        FragmentNode::AwaitBlock(block) => node::await_block::analyze(block, context),
        FragmentNode::KeyBlock(block) => analyze_fragment(&block.fragment, context),
        FragmentNode::SnippetBlock(block) => node::snippet_block::analyze(block, context),

        FragmentNode::RegularElement(element) => {
            node::element::analyze(element.span, &element.attributes, &element.fragment, context);
        }
        FragmentNode::Component(component) => {
            node::element::analyze(
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteElement(element) => {
            node::element::analyze(element.span, &element.attributes, &element.fragment, context);
        }
        FragmentNode::SvelteComponent(component) => {
            node::element::analyze(
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteSelf(component) => {
            node::element::analyze(
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteFragment(component) => {
            node::element::analyze(
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteHead(component) => {
            node::element::analyze(
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteBody(component) => {
            node::element::analyze(
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteWindow(component) => {
            node::element::analyze(
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteDocument(component) => {
            node::element::analyze(
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SvelteBoundary(component) => {
            node::element::analyze(
                component.span,
                &component.attributes,
                &component.fragment,
                context,
            );
        }
        FragmentNode::SlotElement(element) => {
            node::element::analyze(element.span, &element.attributes, &element.fragment, context);
        }
        FragmentNode::TitleElement(element) => {
            node::element::analyze(element.span, &element.attributes, &element.fragment, context);
        }
        FragmentNode::SvelteOptionsRaw(element) => {
            node::element::analyze(element.span, &element.attributes, &element.fragment, context);
        }
    }
}
