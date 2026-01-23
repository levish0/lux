mod context;
pub mod error;
mod parser;

use svelte_ast::root::{Fragment, Root};
use svelte_ast::span::Span;
use winnow::stream::{LocatingSlice, Stateful};

use crate::context::ParseContext;
use crate::error::ParseError;
use crate::parser::ParserInput;
use crate::parser::fragment::document_parser;

#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    pub loose: bool,
}

pub fn parse(source: &str, options: ParseOptions) -> Result<Root, Vec<ParseError>> {
    let ts = detect_typescript(source);
    let context = ParseContext::new(ts, options.loose);

    let input_source = LocatingSlice::new(source);
    let mut parser_input: ParserInput = Stateful {
        input: input_source,
        state: context,
    };

    let nodes = document_parser(&mut parser_input)
        .map_err(|_| vec![ParseError::new(
            error::ErrorKind::UnexpectedEof,
            Span::new(0, source.len()),
            "Failed to parse template",
        )])?;

    let root = Root {
        span: Span::new(0, source.len()),
        fragment: Fragment { nodes },
        css: parser_input.state.css,
        instance: parser_input.state.instance,
        module: parser_input.state.module,
        options: None,
        comments: parser_input.state.comments,
        ts,
    };

    if parser_input.state.errors.is_empty() {
        Ok(root)
    } else {
        Err(parser_input.state.errors)
    }
}

fn detect_typescript(source: &str) -> bool {
    source.contains("<script lang=\"ts\"") || source.contains("<script lang='ts'")
}

#[cfg(test)]
mod tests {
    use super::*;
    use svelte_ast::node::FragmentNode;

    #[test]
    fn parse_empty() {
        let root = parse("", ParseOptions::default()).unwrap();
        assert!(root.fragment.nodes.is_empty());
    }

    #[test]
    fn parse_text_only() {
        let root = parse("hello world", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 1);
        match &root.fragment.nodes[0] {
            FragmentNode::Text(text) => {
                assert_eq!(text.data, "hello world");
                assert_eq!(text.span.start, 0);
                assert_eq!(text.span.end, 11);
            }
            _ => panic!("expected Text node"),
        }
    }

    #[test]
    fn parse_text_and_expression() {
        let root = parse("hello{expr}", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 2);
        match &root.fragment.nodes[0] {
            FragmentNode::Text(text) => {
                assert_eq!(text.data, "hello");
                assert_eq!(text.span.end, 5);
            }
            _ => panic!("expected Text node"),
        }
        match &root.fragment.nodes[1] {
            FragmentNode::ExpressionTag(tag) => {
                assert_eq!(tag.span.start, 5);
                assert_eq!(tag.span.end, 11);
            }
            _ => panic!("expected ExpressionTag"),
        }
    }

    #[test]
    fn parse_void_element() {
        let root = parse("<br>", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 1);
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                assert_eq!(el.name, "br");
                assert!(el.fragment.nodes.is_empty());
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_self_closing_element() {
        let root = parse("<div/>", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 1);
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                assert_eq!(el.name, "div");
                assert!(el.fragment.nodes.is_empty());
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_element_with_text_child() {
        let root = parse("<p>hello</p>", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 1);
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                assert_eq!(el.name, "p");
                assert_eq!(el.span.start, 0);
                assert_eq!(el.span.end, 12);
                assert_eq!(el.fragment.nodes.len(), 1);
                match &el.fragment.nodes[0] {
                    FragmentNode::Text(text) => assert_eq!(text.data, "hello"),
                    _ => panic!("expected Text child"),
                }
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_nested_elements() {
        let root = parse("<div><p>hi</p></div>", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 1);
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(outer) => {
                assert_eq!(outer.name, "div");
                assert_eq!(outer.fragment.nodes.len(), 1);
                match &outer.fragment.nodes[0] {
                    FragmentNode::RegularElement(inner) => {
                        assert_eq!(inner.name, "p");
                        assert_eq!(inner.fragment.nodes.len(), 1);
                        match &inner.fragment.nodes[0] {
                            FragmentNode::Text(t) => assert_eq!(t.data, "hi"),
                            _ => panic!("expected Text"),
                        }
                    }
                    _ => panic!("expected inner RegularElement"),
                }
            }
            _ => panic!("expected outer RegularElement"),
        }
    }

    #[test]
    fn parse_siblings() {
        let root = parse("<p>a</p><p>b</p>", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 2);
        for (i, expected) in ["a", "b"].iter().enumerate() {
            match &root.fragment.nodes[i] {
                FragmentNode::RegularElement(el) => {
                    assert_eq!(el.name, "p");
                    match &el.fragment.nodes[0] {
                        FragmentNode::Text(t) => assert_eq!(&t.data, expected),
                        _ => panic!("expected Text"),
                    }
                }
                _ => panic!("expected RegularElement"),
            }
        }
    }

    #[test]
    fn parse_text_and_element_mixed() {
        let root = parse("before<br>after", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 3);
        match &root.fragment.nodes[0] {
            FragmentNode::Text(t) => assert_eq!(t.data, "before"),
            _ => panic!("expected Text"),
        }
        match &root.fragment.nodes[1] {
            FragmentNode::RegularElement(el) => assert_eq!(el.name, "br"),
            _ => panic!("expected RegularElement"),
        }
        match &root.fragment.nodes[2] {
            FragmentNode::Text(t) => assert_eq!(t.data, "after"),
            _ => panic!("expected Text"),
        }
    }

    #[test]
    fn parse_component() {
        let root = parse("<Button>click</Button>", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 1);
        match &root.fragment.nodes[0] {
            FragmentNode::Component(c) => {
                assert_eq!(c.name, "Button");
                assert_eq!(c.fragment.nodes.len(), 1);
            }
            _ => panic!("expected Component"),
        }
    }

    #[test]
    fn parse_name_loc() {
        let root = parse("<div></div>", ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                assert_eq!(el.name_loc.start, 1);
                assert_eq!(el.name_loc.end, 4);
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_comment() {
        let root = parse("<!-- hello -->", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 1);
        match &root.fragment.nodes[0] {
            FragmentNode::Comment(c) => {
                assert_eq!(c.data, " hello ");
                assert_eq!(c.span.start, 0);
                assert_eq!(c.span.end, 14);
            }
            _ => panic!("expected Comment"),
        }
    }

    #[test]
    fn parse_comment_between_elements() {
        let root = parse("<p>a</p><!-- sep --><p>b</p>", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 3);
        assert!(matches!(&root.fragment.nodes[0], FragmentNode::RegularElement(_)));
        assert!(matches!(&root.fragment.nodes[1], FragmentNode::Comment(_)));
        assert!(matches!(&root.fragment.nodes[2], FragmentNode::RegularElement(_)));
    }

    #[test]
    fn parse_boolean_attribute() {
        let root = parse("<input disabled>", ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                assert_eq!(el.attributes.len(), 1);
                match &el.attributes[0] {
                    svelte_ast::node::AttributeNode::Attribute(attr) => {
                        assert_eq!(attr.name, "disabled");
                        assert!(matches!(attr.value, svelte_ast::attributes::AttributeValue::True));
                    }
                    _ => panic!("expected Attribute"),
                }
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_quoted_attribute() {
        let root = parse(r#"<div class="foo"></div>"#, ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                assert_eq!(el.attributes.len(), 1);
                match &el.attributes[0] {
                    svelte_ast::node::AttributeNode::Attribute(attr) => {
                        assert_eq!(attr.name, "class");
                        match &attr.value {
                            svelte_ast::attributes::AttributeValue::Sequence(seq) => {
                                match &seq[0] {
                                    svelte_ast::attributes::AttributeSequenceValue::Text(t) => {
                                        assert_eq!(t.data, "foo");
                                    }
                                    _ => panic!("expected Text in sequence"),
                                }
                            }
                            _ => panic!("expected Sequence value"),
                        }
                    }
                    _ => panic!("expected Attribute"),
                }
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_multiple_attributes() {
        let root = parse(r#"<div id="app" class="main"></div>"#, ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                assert_eq!(el.attributes.len(), 2);
                match &el.attributes[0] {
                    svelte_ast::node::AttributeNode::Attribute(a) => assert_eq!(a.name, "id"),
                    _ => panic!("expected Attribute"),
                }
                match &el.attributes[1] {
                    svelte_ast::node::AttributeNode::Attribute(a) => assert_eq!(a.name, "class"),
                    _ => panic!("expected Attribute"),
                }
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_self_closing_with_attributes() {
        let root = parse(r#"<input type="text" />"#, ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                assert_eq!(el.name, "input");
                assert_eq!(el.attributes.len(), 1);
                assert!(el.fragment.nodes.is_empty());
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_expression_in_element() {
        let root = parse("<p>{count}</p>", ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                assert_eq!(el.name, "p");
                assert_eq!(el.fragment.nodes.len(), 1);
                assert!(matches!(&el.fragment.nodes[0], FragmentNode::ExpressionTag(_)));
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_expression_with_nested_braces() {
        // Object literal with nested braces
        let root = parse("{({a: 1})}", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 1);
        assert!(matches!(&root.fragment.nodes[0], FragmentNode::ExpressionTag(_)));
    }

    #[test]
    fn parse_expression_method_call() {
        let root = parse("{items.map(x => x.name)}", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 1);
        assert!(matches!(&root.fragment.nodes[0], FragmentNode::ExpressionTag(_)));
    }

    #[test]
    fn parse_mixed_text_expr_element() {
        let root = parse("Count: {count}<br>Done", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 4);
        assert!(matches!(&root.fragment.nodes[0], FragmentNode::Text(_)));
        assert!(matches!(&root.fragment.nodes[1], FragmentNode::ExpressionTag(_)));
        assert!(matches!(&root.fragment.nodes[2], FragmentNode::RegularElement(_)));
        assert!(matches!(&root.fragment.nodes[3], FragmentNode::Text(_)));
    }

    #[test]
    fn parse_if_block() {
        let root = parse("{#if visible}hello{/if}", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 1);
        match &root.fragment.nodes[0] {
            FragmentNode::IfBlock(block) => {
                assert!(!block.elseif);
                assert_eq!(block.consequent.nodes.len(), 1);
                assert!(matches!(&block.consequent.nodes[0], FragmentNode::Text(_)));
                assert!(block.alternate.is_none());
            }
            _ => panic!("expected IfBlock"),
        }
    }

    #[test]
    fn parse_if_else_block() {
        let root = parse("{#if show}yes{:else}no{/if}", ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::IfBlock(block) => {
                assert_eq!(block.consequent.nodes.len(), 1);
                let alt = block.alternate.as_ref().unwrap();
                assert_eq!(alt.nodes.len(), 1);
                match &alt.nodes[0] {
                    FragmentNode::Text(t) => assert_eq!(t.data, "no"),
                    _ => panic!("expected Text in alternate"),
                }
            }
            _ => panic!("expected IfBlock"),
        }
    }

    #[test]
    fn parse_if_else_if_block() {
        let root = parse("{#if a}1{:else if b}2{:else}3{/if}", ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::IfBlock(block) => {
                assert!(!block.elseif);
                let alt = block.alternate.as_ref().unwrap();
                assert_eq!(alt.nodes.len(), 1);
                match &alt.nodes[0] {
                    FragmentNode::IfBlock(nested) => {
                        assert!(nested.elseif);
                        let nested_alt = nested.alternate.as_ref().unwrap();
                        assert_eq!(nested_alt.nodes.len(), 1);
                    }
                    _ => panic!("expected nested IfBlock"),
                }
            }
            _ => panic!("expected IfBlock"),
        }
    }

    #[test]
    fn parse_each_block() {
        let root = parse("{#each items as item}text{/each}", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 1);
        match &root.fragment.nodes[0] {
            FragmentNode::EachBlock(block) => {
                assert_eq!(block.body.nodes.len(), 1);
                assert!(block.index.is_none());
                assert!(block.key.is_none());
                assert!(block.fallback.is_none());
            }
            _ => panic!("expected EachBlock"),
        }
    }

    #[test]
    fn parse_each_with_index() {
        let root = parse("{#each items as item, i}text{/each}", ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::EachBlock(block) => {
                assert_eq!(block.index.as_deref(), Some("i"));
            }
            _ => panic!("expected EachBlock"),
        }
    }

    #[test]
    fn parse_key_block() {
        let root = parse("{#key value}<p>content</p>{/key}", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 1);
        match &root.fragment.nodes[0] {
            FragmentNode::KeyBlock(block) => {
                assert_eq!(block.fragment.nodes.len(), 1);
                assert!(matches!(&block.fragment.nodes[0], FragmentNode::RegularElement(_)));
            }
            _ => panic!("expected KeyBlock"),
        }
    }

    #[test]
    fn parse_await_block() {
        let root = parse(
            "{#await promise}loading{:then value}done{:catch error}failed{/await}",
            ParseOptions::default(),
        )
        .unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::AwaitBlock(block) => {
                assert!(block.pending.is_some());
                assert!(block.then.is_some());
                assert!(block.catch.is_some());
            }
            _ => panic!("expected AwaitBlock"),
        }
    }

    #[test]
    fn parse_snippet_block() {
        let root = parse("{#snippet greeting(name)}<p>hi</p>{/snippet}", ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::SnippetBlock(block) => {
                assert_eq!(block.body.nodes.len(), 1);
                assert_eq!(block.parameters.len(), 1);
            }
            _ => panic!("expected SnippetBlock"),
        }
    }

    #[test]
    fn parse_if_with_element_children() {
        let root = parse("{#if show}<div>hello</div>{/if}", ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::IfBlock(block) => {
                assert_eq!(block.consequent.nodes.len(), 1);
                match &block.consequent.nodes[0] {
                    FragmentNode::RegularElement(el) => {
                        assert_eq!(el.name, "div");
                    }
                    _ => panic!("expected RegularElement"),
                }
            }
            _ => panic!("expected IfBlock"),
        }
    }

    // --- Phase 6: Special tags ---

    #[test]
    fn parse_html_tag() {
        let root = parse("{@html content}", ParseOptions::default()).unwrap();
        assert_eq!(root.fragment.nodes.len(), 1);
        assert!(matches!(&root.fragment.nodes[0], FragmentNode::HtmlTag(_)));
    }

    #[test]
    fn parse_debug_tag() {
        let root = parse("{@debug x, y}", ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::DebugTag(tag) => {
                assert_eq!(tag.identifiers.len(), 2);
            }
            _ => panic!("expected DebugTag"),
        }
    }

    #[test]
    fn parse_debug_tag_empty() {
        let root = parse("{@debug}", ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::DebugTag(tag) => {
                assert_eq!(tag.identifiers.len(), 0);
            }
            _ => panic!("expected DebugTag"),
        }
    }

    #[test]
    fn parse_const_tag() {
        let root = parse("{#each items as item}{@const doubled = item * 2}{doubled}{/each}", ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::EachBlock(block) => {
                assert!(matches!(&block.body.nodes[0], FragmentNode::ConstTag(_)));
            }
            _ => panic!("expected EachBlock"),
        }
    }

    #[test]
    fn parse_render_tag() {
        let root = parse("{@render greeting()}", ParseOptions::default()).unwrap();
        assert!(matches!(&root.fragment.nodes[0], FragmentNode::RenderTag(_)));
    }

    // --- Phase 6: Directives ---

    #[test]
    fn parse_bind_directive() {
        let root = parse(r#"<input bind:value={name}>"#, ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                assert_eq!(el.attributes.len(), 1);
                match &el.attributes[0] {
                    svelte_ast::node::AttributeNode::BindDirective(d) => {
                        assert_eq!(d.name, "value");
                    }
                    _ => panic!("expected BindDirective"),
                }
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_bind_shorthand() {
        let root = parse(r#"<input bind:value>"#, ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                match &el.attributes[0] {
                    svelte_ast::node::AttributeNode::BindDirective(d) => {
                        assert_eq!(d.name, "value");
                    }
                    _ => panic!("expected BindDirective"),
                }
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_on_directive() {
        let root = parse(r#"<button on:click={handler}></button>"#, ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                match &el.attributes[0] {
                    svelte_ast::node::AttributeNode::OnDirective(d) => {
                        assert_eq!(d.name, "click");
                        assert!(d.expression.is_some());
                        assert!(d.modifiers.is_empty());
                    }
                    _ => panic!("expected OnDirective"),
                }
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_on_directive_with_modifiers() {
        let root = parse(r#"<form on:submit|preventDefault={handle}></form>"#, ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                match &el.attributes[0] {
                    svelte_ast::node::AttributeNode::OnDirective(d) => {
                        assert_eq!(d.name, "submit");
                        assert_eq!(d.modifiers.len(), 1);
                    }
                    _ => panic!("expected OnDirective"),
                }
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_class_directive() {
        let root = parse(r#"<div class:active={isActive}></div>"#, ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                match &el.attributes[0] {
                    svelte_ast::node::AttributeNode::ClassDirective(d) => {
                        assert_eq!(d.name, "active");
                    }
                    _ => panic!("expected ClassDirective"),
                }
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_use_directive() {
        let root = parse(r#"<div use:tooltip></div>"#, ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                match &el.attributes[0] {
                    svelte_ast::node::AttributeNode::UseDirective(d) => {
                        assert_eq!(d.name, "tooltip");
                        assert!(d.expression.is_none());
                    }
                    _ => panic!("expected UseDirective"),
                }
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_transition_directive() {
        let root = parse(r#"<div transition:fade></div>"#, ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                match &el.attributes[0] {
                    svelte_ast::node::AttributeNode::TransitionDirective(d) => {
                        assert_eq!(d.name, "fade");
                        assert!(d.intro);
                        assert!(d.outro);
                    }
                    _ => panic!("expected TransitionDirective"),
                }
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_spread_attribute() {
        let root = parse(r#"<div {...props}></div>"#, ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                assert_eq!(el.attributes.len(), 1);
                assert!(matches!(&el.attributes[0], svelte_ast::node::AttributeNode::SpreadAttribute(_)));
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_expression_attribute_value() {
        let root = parse(r#"<div class={expr}></div>"#, ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                match &el.attributes[0] {
                    svelte_ast::node::AttributeNode::Attribute(a) => {
                        assert_eq!(a.name, "class");
                        assert!(matches!(&a.value, svelte_ast::attributes::AttributeValue::Expression(_)));
                    }
                    _ => panic!("expected Attribute"),
                }
            }
            _ => panic!("expected RegularElement"),
        }
    }

    #[test]
    fn parse_mixed_quoted_value() {
        let root = parse(r#"<div class="foo {bar} baz"></div>"#, ParseOptions::default()).unwrap();
        match &root.fragment.nodes[0] {
            FragmentNode::RegularElement(el) => {
                match &el.attributes[0] {
                    svelte_ast::node::AttributeNode::Attribute(a) => {
                        assert_eq!(a.name, "class");
                        match &a.value {
                            svelte_ast::attributes::AttributeValue::Sequence(seq) => {
                                assert_eq!(seq.len(), 3); // "foo ", {bar}, " baz"
                            }
                            _ => panic!("expected Sequence"),
                        }
                    }
                    _ => panic!("expected Attribute"),
                }
            }
            _ => panic!("expected RegularElement"),
        }
    }

    // --- Phase 7: Script parsing ---

    #[test]
    fn parse_script_tag() {
        let root = parse("<script>let x = 1;</script>", ParseOptions::default()).unwrap();
        assert!(root.instance.is_some());
        assert!(root.module.is_none());
        let script = root.instance.unwrap();
        assert_eq!(script.span.start, 0);
        assert_eq!(script.span.end, 27);
        assert!(script.attributes.is_empty());
    }

    #[test]
    fn parse_script_module() {
        let root = parse(r#"<script context="module">export const x = 1;</script>"#, ParseOptions::default()).unwrap();
        assert!(root.instance.is_none());
        assert!(root.module.is_some());
    }

    #[test]
    fn parse_script_with_markup() {
        let root = parse("<script>let count = 0;</script>\n<p>{count}</p>", ParseOptions::default()).unwrap();
        assert!(root.instance.is_some());
        // Fragment should contain the newline and <p> element, but not the script tag
        assert!(!root.fragment.nodes.is_empty());
        let has_script_element = root.fragment.nodes.iter().any(|n| {
            matches!(n, FragmentNode::RegularElement(el) if el.name == "script")
        });
        assert!(!has_script_element, "script should not appear as a fragment element");
    }

    #[test]
    fn parse_script_with_attributes() {
        let root = parse(r#"<script lang="ts">let x: number = 1;</script>"#, ParseOptions { loose: false }).unwrap();
        assert!(root.instance.is_some());
        let script = root.instance.unwrap();
        assert_eq!(script.attributes.len(), 1);
        assert_eq!(script.attributes[0].name, "lang");
    }

    // --- Phase 7: CSS parsing ---

    #[test]
    fn parse_style_tag() {
        let root = parse("<style>div { color: red; }</style>", ParseOptions::default()).unwrap();
        assert!(root.css.is_some());
        let css = root.css.unwrap();
        assert_eq!(css.children.len(), 1);
    }

    #[test]
    fn parse_style_not_in_fragment() {
        let root = parse("<style>p { margin: 0; }</style><p>hi</p>", ParseOptions::default()).unwrap();
        assert!(root.css.is_some());
        // Fragment should only have the <p> element, not the style tag
        let has_style = root.fragment.nodes.iter().any(|n| {
            matches!(n, FragmentNode::RegularElement(el) if el.name == "style")
        });
        assert!(!has_style, "style should not appear as a fragment element");
        assert!(root.fragment.nodes.iter().any(|n| {
            matches!(n, FragmentNode::RegularElement(el) if el.name == "p")
        }));
    }

    #[test]
    fn parse_style_multiple_rules() {
        let root = parse("<style>h1 { font-size: 2em; } p { margin: 0; }</style>", ParseOptions::default()).unwrap();
        let css = root.css.unwrap();
        assert_eq!(css.children.len(), 2);
    }

    #[test]
    fn parse_style_nesting() {
        let root = parse("<style>.parent { color: red; .child { color: blue; } }</style>", ParseOptions::default()).unwrap();
        let css = root.css.unwrap();
        assert_eq!(css.children.len(), 1); // .parent rule
    }

    #[test]
    fn parse_style_nesting_ampersand() {
        let root = parse("<style>.btn { color: red; &:hover { color: blue; } &::after { content: ''; } }</style>", ParseOptions::default()).unwrap();
        let css = root.css.unwrap();
        assert_eq!(css.children.len(), 1); // .btn rule
    }

    #[test]
    fn parse_style_nesting_pseudo_class() {
        let root = parse("<style>div { :global(.foo) { color: red; } }</style>", ParseOptions::default()).unwrap();
        let css = root.css.unwrap();
        assert_eq!(css.children.len(), 1);
    }

    #[test]
    fn parse_script_and_style() {
        let root = parse(
            "<script>let x = 1;</script>\n<p>hi</p>\n<style>p { color: blue; }</style>",
            ParseOptions::default(),
        ).unwrap();
        assert!(root.instance.is_some());
        assert!(root.css.is_some());
        // Fragment should only have newlines and <p>
        assert!(root.fragment.nodes.iter().any(|n| {
            matches!(n, FragmentNode::RegularElement(el) if el.name == "p")
        }));
    }
}
