use oxc_ast::ast::{
    Expression, VariableDeclaration, VariableDeclarationKind, VariableDeclarator,
};

use svelte_ast::node::FragmentNode;
use svelte_ast::span::Span;
use svelte_ast::tags::{ConstTag, DebugTag, HtmlTag, RenderTag};

use crate::error::ErrorKind;
use crate::parser::patterns::REGEX_WHITESPACE_WITH_CLOSING_CURLY_BRACE;
use crate::parser::read::context::read_pattern;
use crate::parser::read::expression::read_expression;
use crate::parser::Parser;

use super::{dummy_binding_pattern, expression_end, pattern_span, skip_to_closing_brace};

/// `{@...}` — special tags (html, debug, const, render)
pub fn special<'a>(parser: &mut Parser<'a>) {
    let mut start = parser.index;
    while start > 0 && parser.template.as_bytes()[start] != b'{' {
        start -= 1;
    }

    if parser.eat("html") {
        parser.require_whitespace();

        let expression = read_expression(parser);

        parser.allow_whitespace();
        parser.eat_required("}");

        parser.append(FragmentNode::HtmlTag(HtmlTag {
            span: Span::new(start, parser.index),
            expression,
        }));

        return;
    }

    if parser.eat("debug") {
        // {@debug} with no args means "debug all"
        if parser.read(&REGEX_WHITESPACE_WITH_CLOSING_CURLY_BRACE).is_some() {
            parser.append(FragmentNode::DebugTag(DebugTag {
                span: Span::new(start, parser.index),
                identifiers: Vec::new(),
            }));
            return;
        }

        let expression = read_expression(parser);

        // Extract identifiers from expression (could be SequenceExpression)
        let identifiers = match expression {
            Expression::SequenceExpression(seq) => {
                let seq = seq.unbox();
                seq.expressions.into_iter().collect()
            }
            other => {
                vec![other]
            }
        };

        parser.allow_whitespace();
        parser.eat_required("}");

        parser.append(FragmentNode::DebugTag(DebugTag {
            span: Span::new(start, parser.index),
            identifiers,
        }));

        return;
    }

    if parser.eat("const") {
        parser.require_whitespace();

        let id = read_pattern(parser);
        parser.allow_whitespace();

        parser.eat_required("=");
        parser.allow_whitespace();

        let init = read_expression(parser);
        parser.allow_whitespace();

        parser.eat_required("}");

        // Build VariableDeclaration
        let id_span = pattern_span(&id);
        let init_end = expression_end(&init);

        let declarator = VariableDeclarator {
            span: oxc_span::Span::new(id_span.0, init_end),
            kind: VariableDeclarationKind::Const,
            id: id.unwrap_or_else(|| dummy_binding_pattern(parser, start)),
            init: Some(init),
            definite: false,
            type_annotation: None,
        };

        let mut declarations = oxc_allocator::Vec::new_in(parser.allocator);
        declarations.push(declarator);

        let declaration = VariableDeclaration {
            span: oxc_span::Span::new((start + 2) as u32, (parser.index - 1) as u32),
            kind: VariableDeclarationKind::Const,
            declarations,
            declare: false,
        };

        parser.append(FragmentNode::ConstTag(ConstTag {
            span: Span::new(start, parser.index),
            declaration,
        }));

        return;
    }

    if parser.eat("render") {
        parser.require_whitespace();

        let expression = read_expression(parser);

        parser.allow_whitespace();
        parser.eat_required("}");

        parser.append(FragmentNode::RenderTag(RenderTag {
            span: Span::new(start, parser.index),
            expression,
        }));

        return;
    }

    if !parser.loose {
        parser.error(
            ErrorKind::ExpectedToken,
            parser.index,
            "Expected tag type (html, debug, const, or render)".to_string(),
        );
    }
    skip_to_closing_brace(parser);
}
