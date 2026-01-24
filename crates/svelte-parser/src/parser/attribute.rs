use svelte_ast::JsNode;
use svelte_ast::attributes::{
    AnimateDirective, Attribute, AttributeSequenceValue, AttributeValue, BindDirective,
    ClassDirective, EventModifier, LetDirective, OnDirective, SpreadAttribute, StyleDirective,
    TransitionDirective, TransitionModifier, UseDirective,
};
use svelte_ast::node::AttributeNode;
use svelte_ast::span::{SourceLocation, Span};
use svelte_ast::tags::{AttachTag, ExpressionTag};
use svelte_ast::text::Text;
use winnow::Result as ParseResult;
use winnow::combinator::{dispatch, opt, peek};
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{any, literal, take_while};

use super::ParserInput;
use super::bracket::read_until_close_brace;
use super::html_entities::decode_character_references;
use super::oxc_parse::{parse_expression, parse_expression_with_comments};

/// Parse a single attribute, directive, or spread.
pub fn attribute_parser(parser_input: &mut ParserInput) -> ParseResult<AttributeNode> {
    dispatch! {peek(any);
        '{' => spread_or_shorthand_parser,
        _ => named_attribute_parser,
    }
    .parse_next(parser_input)
}

/// Parse `{...expr}` spread, `{@attach expr}`, or `{name}` shorthand.
fn spread_or_shorthand_parser(parser_input: &mut ParserInput) -> ParseResult<AttributeNode> {
    let start = parser_input.current_token_start();
    literal("{").parse_next(parser_input)?;

    // Check for @attach: {@attach ...
    if opt(literal("@attach")).parse_next(parser_input)?.is_some() {
        take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
        let expr_offset = parser_input.current_token_start() as u32;
        let expr_text = read_until_close_brace(parser_input)?;
        literal("}").parse_next(parser_input)?;
        let end = parser_input.previous_token_end();

        let expression = parse_expression(expr_text, parser_input.state.ts, expr_offset)?;

        return Ok(AttributeNode::AttachTag(AttachTag {
            span: Span::new(start, end),
            expression,
        }));
    }

    // Check for spread: {...
    if opt(literal("...")).parse_next(parser_input)?.is_some() {
        let expr_offset = parser_input.current_token_start() as u32;
        let expr_text = read_until_close_brace(parser_input)?;
        literal("}").parse_next(parser_input)?;
        let end = parser_input.previous_token_end();

        let expression = parse_expression(expr_text, parser_input.state.ts, expr_offset)?;

        return Ok(AttributeNode::SpreadAttribute(SpreadAttribute {
            span: Span::new(start, end),
            expression,
        }));
    }

    // Shorthand: {name} is equivalent to name={name}
    let name: &str = take_while(0.., |c: char| {
        c.is_ascii_alphanumeric() || c == '_' || c == '$'
    })
    .parse_next(parser_input)?;

    if name.is_empty() && !parser_input.state.loose {
        return Err(winnow::error::ContextError::new());
    }

    // In loose mode with empty name, the expression content might be invalid
    if name.is_empty() {
        let expr_offset = parser_input.current_token_start() as u32;
        let _remaining = read_until_close_brace(parser_input)?;
        literal("}").parse_next(parser_input)?;
        let end = parser_input.previous_token_end();

        let expression = JsNode(serde_json::json!({
            "type": "Identifier",
            "name": "",
            "start": expr_offset,
            "end": expr_offset
        }));

        return Ok(AttributeNode::Attribute(Attribute {
            span: Span::new(start, end),
            name: String::new(),
            name_loc: Some(
                parser_input
                    .state
                    .locator
                    .locate_span(expr_offset as usize, expr_offset as usize),
            ),
            value: AttributeValue::Expression(ExpressionTag {
                span: Span::new(expr_offset as usize, expr_offset as usize),
                expression,
                force_expression_loc: true,
            }),
        }));
    }

    let name_start_pos = start + 1; // after {
    let name_end_pos = name_start_pos + name.len();

    literal("}").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    let name_loc = parser_input
        .state
        .locator
        .locate_span(name_start_pos, name_end_pos);

    let expression = JsNode(serde_json::json!({
        "type": "Identifier",
        "name": name,
        "start": name_start_pos,
        "end": name_end_pos
    }));

    Ok(AttributeNode::Attribute(Attribute {
        span: Span::new(start, end),
        name: name.to_string(),
        name_loc: Some(name_loc),
        value: AttributeValue::Expression(ExpressionTag {
            span: Span::new(name_start_pos, name_end_pos),
            expression,
            force_expression_loc: true,
        }),
    }))
}

/// Parse a named attribute or directive.
fn named_attribute_parser(parser_input: &mut ParserInput) -> ParseResult<AttributeNode> {
    let start = parser_input.current_token_start();

    // Read the full attribute name (including : for directives and | for modifiers)
    let name_start = parser_input.current_token_start();
    let full_name: &str = take_while(1.., |c: char| {
        c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':' | '.' | '|' | '$')
    })
    .parse_next(parser_input)?;
    let name_end = parser_input.previous_token_end();
    let name_loc = parser_input.state.locator.locate_span(name_start, name_end);

    // Check for directive prefix
    if let Some(colon_pos) = full_name.find(':') {
        let prefix = &full_name[..colon_pos];
        let rest = &full_name[colon_pos + 1..];

        match prefix {
            "bind" => return parse_bind_directive(parser_input, start, rest, name_loc),
            "on" => return parse_on_directive(parser_input, start, rest, name_loc),
            "class" => return parse_class_directive(parser_input, start, rest, name_loc),
            "style" => return parse_style_directive(parser_input, start, rest, name_loc),
            "transition" => {
                return parse_transition_directive(parser_input, start, rest, name_loc, true, true);
            }
            "in" => {
                return parse_transition_directive(
                    parser_input,
                    start,
                    rest,
                    name_loc,
                    true,
                    false,
                );
            }
            "out" => {
                return parse_transition_directive(
                    parser_input,
                    start,
                    rest,
                    name_loc,
                    false,
                    true,
                );
            }
            "animate" => return parse_animate_directive(parser_input, start, rest, name_loc),
            "use" => return parse_use_directive(parser_input, start, rest, name_loc),
            "let" => return parse_let_directive(parser_input, start, rest, name_loc),
            _ => {} // Not a directive, treat as normal attribute with colon in name
        }
    }

    // Regular attribute
    let has_value = opt(literal("=")).parse_next(parser_input)?.is_some();
    let value = if has_value {
        parse_attribute_value(parser_input)?
    } else {
        AttributeValue::True
    };

    let end = parser_input.previous_token_end();

    Ok(AttributeNode::Attribute(Attribute {
        span: Span::new(start, end),
        name: full_name.to_string(),
        name_loc: Some(name_loc),
        value,
    }))
}

// --- Directive parsers ---

fn parse_bind_directive(
    parser_input: &mut ParserInput,
    start: usize,
    name: &str,
    name_loc: SourceLocation,
) -> ParseResult<AttributeNode> {
    let (expr, leading_comments) = parse_bind_directive_value(parser_input, name)?;
    let end = parser_input.previous_token_end();

    Ok(AttributeNode::BindDirective(BindDirective {
        span: Span::new(start, end),
        name: name.to_string(),
        name_loc: Some(name_loc),
        expression: expr,
        leading_comments,
    }))
}

/// Parse bind directive value with comment collection.
fn parse_bind_directive_value(
    parser_input: &mut ParserInput,
    name: &str,
) -> ParseResult<(JsNode, Vec<svelte_ast::text::JsComment>)> {
    if opt(literal("=")).parse_next(parser_input)?.is_none() {
        let expr = JsNode(serde_json::json!({
            "type": "Identifier",
            "name": name,
            "start": 0,
            "end": 0
        }));
        return Ok((expr, vec![]));
    }

    literal("{").parse_next(parser_input)?;
    let expr_offset = parser_input.current_token_start() as u32;
    let expr_text = read_until_close_brace(parser_input)?;
    literal("}").parse_next(parser_input)?;

    let loose = parser_input.state.loose;
    match parse_expression_with_comments(expr_text, parser_input.state.ts, expr_offset) {
        Ok((expr, comments)) => Ok((expr, comments)),
        Err(_) if loose => {
            let expr = super::expression::make_empty_ident(expr_text, expr_offset);
            Ok((expr, vec![]))
        }
        Err(e) => Err(e),
    }
}

fn parse_on_directive(
    parser_input: &mut ParserInput,
    start: usize,
    name_and_modifiers: &str,
    name_loc: SourceLocation,
) -> ParseResult<AttributeNode> {
    let parts: Vec<&str> = name_and_modifiers.split('|').collect();
    let name = parts[0];
    let modifiers: Vec<EventModifier> = parts[1..]
        .iter()
        .filter_map(|m| parse_event_modifier(m))
        .collect();

    let expression = parse_optional_directive_value(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(AttributeNode::OnDirective(OnDirective {
        span: Span::new(start, end),
        name: name.to_string(),
        name_loc: Some(name_loc),
        expression,
        modifiers,
    }))
}

fn parse_class_directive(
    parser_input: &mut ParserInput,
    start: usize,
    name_and_modifiers: &str,
    name_loc: SourceLocation,
) -> ParseResult<AttributeNode> {
    let parts: Vec<&str> = name_and_modifiers.split('|').collect();
    let name = parts[0];
    let modifiers: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

    let expression = parse_optional_directive_value(parser_input)?;
    let end = parser_input.previous_token_end();

    let expr = expression.unwrap_or_else(|| {
        JsNode(serde_json::json!({
            "type": "Identifier",
            "name": name,
            "start": 0,
            "end": 0
        }))
    });

    Ok(AttributeNode::ClassDirective(ClassDirective {
        span: Span::new(start, end),
        name: name.to_string(),
        name_loc: Some(name_loc),
        expression: expr,
        modifiers,
    }))
}

fn parse_style_directive(
    parser_input: &mut ParserInput,
    start: usize,
    name_and_modifiers: &str,
    name_loc: SourceLocation,
) -> ParseResult<AttributeNode> {
    let parts: Vec<&str> = name_and_modifiers.split('|').collect();
    let name = parts[0];
    let modifiers = parts[1..]
        .iter()
        .filter_map(|m| {
            if *m == "important" {
                Some(svelte_ast::attributes::StyleModifier::Important)
            } else {
                None
            }
        })
        .collect();

    let has_value = opt(literal("=")).parse_next(parser_input)?.is_some();
    let value = if has_value {
        parse_attribute_value(parser_input)?
    } else {
        AttributeValue::True
    };
    let end = parser_input.previous_token_end();

    Ok(AttributeNode::StyleDirective(StyleDirective {
        span: Span::new(start, end),
        name: name.to_string(),
        name_loc: Some(name_loc),
        value,
        modifiers,
    }))
}

fn parse_transition_directive(
    parser_input: &mut ParserInput,
    start: usize,
    name_and_modifiers: &str,
    name_loc: SourceLocation,
    intro: bool,
    outro: bool,
) -> ParseResult<AttributeNode> {
    let parts: Vec<&str> = name_and_modifiers.split('|').collect();
    let name = parts[0];
    let modifiers: Vec<TransitionModifier> = parts[1..]
        .iter()
        .filter_map(|m| match *m {
            "local" => Some(TransitionModifier::Local),
            "global" => Some(TransitionModifier::Global),
            _ => None,
        })
        .collect();

    let expression = parse_optional_directive_value(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(AttributeNode::TransitionDirective(TransitionDirective {
        span: Span::new(start, end),
        name: name.to_string(),
        name_loc: Some(name_loc),
        expression,
        modifiers,
        intro,
        outro,
    }))
}

fn parse_animate_directive(
    parser_input: &mut ParserInput,
    start: usize,
    name_and_modifiers: &str,
    name_loc: SourceLocation,
) -> ParseResult<AttributeNode> {
    let parts: Vec<&str> = name_and_modifiers.split('|').collect();
    let name = parts[0];
    let modifiers: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

    let expression = parse_optional_directive_value(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(AttributeNode::AnimateDirective(AnimateDirective {
        span: Span::new(start, end),
        name: name.to_string(),
        name_loc: Some(name_loc),
        expression,
        modifiers,
    }))
}

fn parse_use_directive(
    parser_input: &mut ParserInput,
    start: usize,
    name_and_modifiers: &str,
    name_loc: SourceLocation,
) -> ParseResult<AttributeNode> {
    let parts: Vec<&str> = name_and_modifiers.split('|').collect();
    let name = parts[0];
    let modifiers: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

    let expression = parse_optional_directive_value(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(AttributeNode::UseDirective(UseDirective {
        span: Span::new(start, end),
        name: name.to_string(),
        name_loc: Some(name_loc),
        expression,
        modifiers,
    }))
}

fn parse_let_directive(
    parser_input: &mut ParserInput,
    start: usize,
    name_and_modifiers: &str,
    name_loc: SourceLocation,
) -> ParseResult<AttributeNode> {
    let parts: Vec<&str> = name_and_modifiers.split('|').collect();
    let name = parts[0];
    let modifiers: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

    let expression = parse_optional_directive_value(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(AttributeNode::LetDirective(LetDirective {
        span: Span::new(start, end),
        name: name.to_string(),
        name_loc: Some(name_loc),
        expression,
        modifiers,
    }))
}

// --- Value parsing helpers ---

/// Parse optional `={expr}` value for directives.
fn parse_optional_directive_value(
    parser_input: &mut ParserInput,
) -> ParseResult<Option<JsNode>> {
    if opt(literal("=")).parse_next(parser_input)?.is_none() {
        return Ok(None);
    }

    // Must be {expr}
    literal("{").parse_next(parser_input)?;
    let expr_offset = parser_input.current_token_start() as u32;
    let expr_text = read_until_close_brace(parser_input)?;
    literal("}").parse_next(parser_input)?;

    let expr = parse_expression(expr_text, parser_input.state.ts, expr_offset)?;
    Ok(Some(expr))
}

fn parse_attribute_value(parser_input: &mut ParserInput) -> ParseResult<AttributeValue> {
    dispatch! {peek(any);
        '"' => |input: &mut ParserInput| parse_quoted_value(input, '"'),
        '\'' => |input: &mut ParserInput| parse_quoted_value(input, '\''),
        '{' => parse_expression_value,
        _ => parse_unquoted_value,
    }
    .parse_next(parser_input)
}

fn parse_expression_value(parser_input: &mut ParserInput) -> ParseResult<AttributeValue> {
    let start = parser_input.current_token_start();
    literal("{").parse_next(parser_input)?;
    let expr_offset = parser_input.current_token_start() as u32;
    let expr_text = read_until_close_brace(parser_input)?;
    literal("}").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();
    let loose = parser_input.state.loose;
    let expression = super::expression::parse_expression_or_loose(
        expr_text,
        parser_input.state.ts,
        expr_offset,
        loose,
    )?;
    Ok(AttributeValue::Expression(ExpressionTag {
        span: Span::new(start, end),
        expression,
        force_expression_loc: false,
    }))
}

fn parse_unquoted_value(parser_input: &mut ParserInput) -> ParseResult<AttributeValue> {
    let val_start = parser_input.current_token_start();
    let text: &str = take_while(1.., |c: char| {
        !c.is_ascii_whitespace() && !matches!(c, '>' | '/' | '=' | '{' | '}' | '<')
    })
    .parse_next(parser_input)?;
    let val_end = parser_input.previous_token_end();
    Ok(AttributeValue::Sequence(vec![
        AttributeSequenceValue::Text(Text {
            span: Span::new(val_start, val_end),
            data: decode_character_references(text, true),
            raw: text.to_string(),
        }),
    ]))
}

/// Parse quoted attribute value with potential embedded expressions.
fn parse_quoted_value(parser_input: &mut ParserInput, quote: char) -> ParseResult<AttributeValue> {
    any.parse_next(parser_input)?; // consume opening quote

    let mut sequence: Vec<AttributeSequenceValue> = Vec::new();
    let mut text_buf = String::new();
    let mut text_start = parser_input.current_token_start();

    loop {
        let c: char = peek(any).parse_next(parser_input)?;
        if c == quote {
            // Flush text buffer
            if !text_buf.is_empty() {
                let text_end = parser_input.current_token_start();
                sequence.push(AttributeSequenceValue::Text(Text {
                    span: Span::new(text_start, text_end),
                    data: decode_character_references(&text_buf, true),
                    raw: text_buf.clone(),
                }));
                text_buf.clear();
            }
            any.parse_next(parser_input)?; // consume closing quote
            break;
        } else if c == '{' && !parser_input.state.text_only_attributes {
            // Flush text buffer
            if !text_buf.is_empty() {
                let text_end = parser_input.current_token_start();
                sequence.push(AttributeSequenceValue::Text(Text {
                    span: Span::new(text_start, text_end),
                    data: decode_character_references(&text_buf, true),
                    raw: text_buf.clone(),
                }));
                text_buf.clear();
            }
            // Parse expression
            let expr_start = parser_input.current_token_start();
            literal("{").parse_next(parser_input)?;
            let expr_offset = parser_input.current_token_start() as u32;
            let expr_text = read_until_close_brace(parser_input)?;
            literal("}").parse_next(parser_input)?;
            let expr_end = parser_input.previous_token_end();
            let expression = parse_expression(expr_text, parser_input.state.ts, expr_offset)?;
            sequence.push(AttributeSequenceValue::ExpressionTag(ExpressionTag {
                span: Span::new(expr_start, expr_end),
                expression,
                force_expression_loc: false,
            }));
            text_start = parser_input.current_token_start();
        } else {
            any.parse_next(parser_input)?;
            text_buf.push(c);
        }
    }

    Ok(AttributeValue::Sequence(sequence))
}

// --- Modifier parsing ---

fn parse_event_modifier(s: &str) -> Option<EventModifier> {
    match s {
        "capture" => Some(EventModifier::Capture),
        "nonpassive" => Some(EventModifier::Nonpassive),
        "once" => Some(EventModifier::Once),
        "passive" => Some(EventModifier::Passive),
        "preventDefault" => Some(EventModifier::PreventDefault),
        "self" => Some(EventModifier::Self_),
        "stopImmediatePropagation" => Some(EventModifier::StopImmediatePropagation),
        "stopPropagation" => Some(EventModifier::StopPropagation),
        "trusted" => Some(EventModifier::Trusted),
        _ => None,
    }
}
