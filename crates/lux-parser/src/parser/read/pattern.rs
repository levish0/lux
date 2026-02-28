use oxc_ast::ast::{BindingPattern, Expression};
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use winnow::Result;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::take;

use crate::input::Input;
use crate::parser::utils::scanner::scan_expression_boundary;
use crate::parser::utils::span_offset::shift_binding_pattern_spans;

/// Read a JS/TS binding pattern up to top-level template delimiters.
///
/// We parse by wrapping the source as an arrow function parameter:
/// `(<pattern>) => {}` and extracting the first parameter pattern.
pub fn read_binding_pattern_until<'a>(
    input: &mut Input<'a>,
    extra_stops: &[u8],
) -> Result<BindingPattern<'a>> {
    let offset = input.current_token_start() as u32;
    let allocator = input.state.allocator;
    let ts = input.state.ts;
    let remaining: &str = &input.input;

    let end = scan_expression_boundary(remaining, extra_stops).ok_or_else(ContextError::new)?;
    let pattern_source = remaining[..end].trim_end();

    if pattern_source.is_empty() {
        return Err(ContextError::new());
    }

    let wrapped_owned = format!("({pattern_source})=>{{}}");
    let wrapped = allocator.alloc_str(&wrapped_owned);
    let source_type = make_source_type(ts);

    let expression = OxcParser::new(allocator, wrapped, source_type)
        .parse_expression()
        .map_err(|_| ContextError::new())?;

    let mut pattern = extract_parameter_pattern(expression).ok_or_else(ContextError::new)?;

    let _ = take(pattern_source.len()).parse_next(input)?;
    // Wrapped source prepends `(` before pattern.
    shift_binding_pattern_spans(&mut pattern, offset.saturating_sub(1));

    Ok(pattern)
}

fn extract_parameter_pattern<'a>(expression: Expression<'a>) -> Option<BindingPattern<'a>> {
    let Expression::ArrowFunctionExpression(arrow) = expression else {
        return None;
    };

    let mut params = arrow.unbox().params.unbox();

    if let Some(rest) = params.rest.take() {
        return Some(rest.unbox().rest.argument);
    }

    let mut items = params.items.into_iter();
    let param = items.next()?;

    if items.next().is_some() {
        return None;
    }

    Some(param.pattern)
}

fn make_source_type(ts: bool) -> SourceType {
    if ts {
        SourceType::ts()
    } else {
        SourceType::mjs()
    }
}
