use crate::error::ErrorKind;
use crate::parser::read::context::read_pattern;
use crate::parser::read::expression::read_expression;
use crate::parser::{AwaitPhase, ParseError, Parser, StackFrame};

use super::skip_to_closing_brace;

/// `{:...}` — continuation (else, then, catch)
pub fn next<'a>(parser: &mut Parser<'a>) -> Result<(), ParseError> {
    let start = parser.index - 1;

    // Determine what type of block we're in
    let current_type = match parser.current() {
        Some(StackFrame::IfBlock { .. }) => "if",
        Some(StackFrame::EachBlock { .. }) => "each",
        Some(StackFrame::AwaitBlock { .. }) => "await",
        _ => {
            if !parser.loose {
                return Err(parser.error(
                    ErrorKind::BlockUnexpectedClose,
                    start,
                    "Block continuation not allowed here".to_string(),
                ));
            }
            skip_to_closing_brace(parser);
            return Ok(());
        }
    };

    match current_type {
        "if" => next_if(parser, start)?,
        "each" => next_each(parser, start)?,
        "await" => next_await(parser, start)?,
        _ => {
            if !parser.loose {
                return Err(parser.error(
                    ErrorKind::BlockUnexpectedClose,
                    start,
                    "Block continuation not allowed here".to_string(),
                ));
            }
            skip_to_closing_brace(parser);
        }
    }

    Ok(())
}

/// Handle {:else} and {:else if} for IfBlock
fn next_if<'a>(parser: &mut Parser<'a>, start: usize) -> Result<(), ParseError> {
    if !parser.eat("else") {
        if !parser.loose {
            return Err(parser.error(
                ErrorKind::ExpectedToken,
                start,
                "Expected {:else} or {:else if}".to_string(),
            ));
        }
        skip_to_closing_brace(parser);
        return Ok(());
    }

    if parser.eat("if") {
        // {:elseif} is invalid — must be {:else if}
        if !parser.loose {
            return Err(parser.error(
                ErrorKind::ExpectedToken,
                start,
                "Use {:else if} not {:elseif}".to_string(),
            ));
        }
    }

    parser.allow_whitespace();

    // Pop current fragment (consequent of current IfBlock)
    let consequent_nodes = parser.fragments.pop().unwrap_or_default();

    // Set consequent on current IfBlock
    if let Some(StackFrame::IfBlock { consequent, .. }) = parser.stack.last_mut() {
        *consequent = Some(consequent_nodes);
    }

    // {:else if ...}
    if parser.eat("if") {
        parser.require_whitespace()?;

        let test = read_expression(parser)?;

        parser.allow_whitespace();
        parser.eat_required("}")?;

        let mut elseif_start = start;
        while elseif_start > 0 && parser.template.as_bytes()[elseif_start] != b'{' {
            elseif_start -= 1;
        }

        // Push placeholder fragment for parent (maintains stack alignment)
        parser.fragments.push(Vec::new());

        // Push child IfBlock
        parser.stack.push(StackFrame::IfBlock {
            start: elseif_start,
            elseif: true,
            test,
            consequent: None,
        });
        parser.fragments.push(Vec::new());
    } else {
        // {:else}
        parser.allow_whitespace();
        parser.eat_required("}")?;

        // Push alternate fragment
        parser.fragments.push(Vec::new());
    }

    Ok(())
}

/// Handle {:else} for EachBlock
fn next_each<'a>(parser: &mut Parser<'a>, start: usize) -> Result<(), ParseError> {
    if !parser.eat("else") {
        if !parser.loose {
            return Err(parser.error(
                ErrorKind::ExpectedToken,
                start,
                "Expected {:else}".to_string(),
            ));
        }
        skip_to_closing_brace(parser);
        return Ok(());
    }

    parser.allow_whitespace();
    parser.eat_required("}")?;

    // Pop body fragment, store in EachBlock
    let body_nodes = parser.fragments.pop().unwrap_or_default();
    if let Some(StackFrame::EachBlock { body, .. }) = parser.stack.last_mut() {
        *body = Some(body_nodes);
    }

    // Push fallback fragment
    parser.fragments.push(Vec::new());

    Ok(())
}

/// Handle {:then} and {:catch} for AwaitBlock
fn next_await<'a>(parser: &mut Parser<'a>, start: usize) -> Result<(), ParseError> {
    let loose = parser.loose;

    if parser.eat("then") {
        // {:then value}
        let new_value = if !parser.eat("}") {
            parser.require_whitespace()?;
            let v = read_pattern(parser);
            parser.allow_whitespace();
            parser.eat_required("}")?;
            v
        } else {
            None
        };

        // Pop current phase fragment, store appropriately
        let phase_nodes = parser.fragments.pop().unwrap_or_default();
        let mut duplicate = false;
        if let Some(StackFrame::AwaitBlock {
            pending,
            then,
            phase,
            value,
            ..
        }) = parser.stack.last_mut()
        {
            if *phase == AwaitPhase::Then && then.is_some() {
                duplicate = true;
            }
            match *phase {
                AwaitPhase::Pending => *pending = Some(phase_nodes),
                AwaitPhase::Then => *then = Some(phase_nodes),
                AwaitPhase::Catch => {}
            }
            *phase = AwaitPhase::Then;
            *value = new_value;
        }

        if duplicate && !loose {
            return Err(parser.error(
                ErrorKind::ExpectedToken,
                start,
                "Duplicate {:then} clause".to_string(),
            ));
        }

        // Push new fragment for then phase
        parser.fragments.push(Vec::new());
    } else if parser.eat("catch") {
        // {:catch error}
        let new_error = if !parser.eat("}") {
            parser.require_whitespace()?;
            let e = read_pattern(parser);
            parser.allow_whitespace();
            parser.eat_required("}")?;
            e
        } else {
            None
        };

        // Pop current phase fragment, store appropriately
        let phase_nodes = parser.fragments.pop().unwrap_or_default();
        let mut duplicate = false;
        if let Some(StackFrame::AwaitBlock {
            pending,
            then,
            phase,
            error,
            ..
        }) = parser.stack.last_mut()
        {
            if *phase == AwaitPhase::Catch {
                duplicate = true;
            }
            match *phase {
                AwaitPhase::Pending => *pending = Some(phase_nodes),
                AwaitPhase::Then => *then = Some(phase_nodes),
                AwaitPhase::Catch => {}
            }
            *phase = AwaitPhase::Catch;
            *error = new_error;
        }

        if duplicate && !loose {
            return Err(parser.error(
                ErrorKind::ExpectedToken,
                start,
                "Duplicate {:catch} clause".to_string(),
            ));
        }

        // Push new fragment for catch phase
        parser.fragments.push(Vec::new());
    } else {
        if !loose {
            return Err(parser.error(
                ErrorKind::ExpectedToken,
                start,
                "Expected {:then ...} or {:catch ...}".to_string(),
            ));
        }
        skip_to_closing_brace(parser);
    }

    Ok(())
}
