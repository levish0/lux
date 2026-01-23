mod context;
pub mod error;
mod parser;

use svelte_ast::root::{Fragment, Root};
use svelte_ast::span::Span;
use winnow::prelude::*;
use winnow::stream::{LocatingSlice, Stateful};

use crate::context::ParseContext;
use crate::error::ParseError;
use crate::parser::ParserInput;
use crate::parser::fragment::fragment_parser;

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

    let nodes = fragment_parser
        .parse_next(&mut parser_input)
        .map_err(|_| vec![ParseError::new(
            error::ErrorKind::UnexpectedEof,
            Span::new(0, source.len()),
            "Failed to parse template",
        )])?;

    let root = Root {
        span: Span::new(0, source.len()),
        fragment: Fragment { nodes },
        css: None,
        instance: None,
        module: None,
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
}
