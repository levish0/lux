use swc_common::input::StringInput;
use swc_common::BytePos;
use swc_ecma_ast as swc;
use swc_ecma_parser::{EsSyntax, Syntax, TsSyntax};
use winnow::combinator::peek;
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::any;
use winnow::Result as ParseResult;
use super::ParserInput;

/// Read a JS/TS expression enclosed in `{ ... }`.
/// Consumes from `{` to matching `}`, parses with SWC.
pub fn read_expression(parser_input: &mut ParserInput) -> ParseResult<Box<swc::Expr>> {
    // Position of `{` - expression text starts at offset + 1
    let brace_offset = parser_input.current_token_start();
    let expr_str = scan_expression_text(parser_input)?;
    let ts = parser_input.state.ts;
    swc_parse_expression(&expr_str, ts, (brace_offset + 1) as u32)
}

/// Consume `{` ... `}` with brace matching, return the inner text.
pub fn scan_expression_text(parser_input: &mut ParserInput) -> ParseResult<String> {
    let c: char = any.parse_next(parser_input)?;
    debug_assert_eq!(c, '{');

    let mut result = String::new();
    let mut depth: u32 = 1;

    while depth > 0 {
        let c: char = any.parse_next(parser_input)?;
        match c {
            '{' => {
                depth += 1;
                result.push(c);
            }
            '}' => {
                depth -= 1;
                if depth > 0 {
                    result.push(c);
                }
            }
            '"' | '\'' => {
                result.push(c);
                collect_string(parser_input, c, &mut result)?;
            }
            '`' => {
                result.push(c);
                collect_template_literal(parser_input, &mut result)?;
            }
            '/' => {
                let next: ParseResult<char> = peek(any).parse_next(parser_input);
                let next = next.ok();
                match next {
                    Some('/') => {
                        result.push(c);
                        collect_line_comment(parser_input, &mut result)?;
                    }
                    Some('*') => {
                        result.push(c);
                        collect_block_comment(parser_input, &mut result)?;
                    }
                    _ => result.push(c),
                }
            }
            _ => result.push(c),
        }
    }

    Ok(result)
}

fn collect_string(parser_input: &mut ParserInput, quote: char, out: &mut String) -> ParseResult<()> {
    loop {
        let c: char = any.parse_next(parser_input)?;
        out.push(c);
        if c == quote {
            return Ok(());
        }
        if c == '\\' {
            let escaped: char = any.parse_next(parser_input)?;
            out.push(escaped);
        }
    }
}

fn collect_template_literal(parser_input: &mut ParserInput, out: &mut String) -> ParseResult<()> {
    loop {
        let c: char = any.parse_next(parser_input)?;
        out.push(c);
        match c {
            '`' => return Ok(()),
            '\\' => {
                let escaped: char = any.parse_next(parser_input)?;
                out.push(escaped);
            }
            '$' => {
                let next: ParseResult<char> = peek(any).parse_next(parser_input);
                let next = next.ok();
                if next == Some('{') {
                    let brace: char = any.parse_next(parser_input)?;
                    out.push(brace);
                    collect_template_expr(parser_input, out)?;
                }
            }
            _ => {}
        }
    }
}

fn collect_template_expr(parser_input: &mut ParserInput, out: &mut String) -> ParseResult<()> {
    let mut depth: u32 = 1;
    while depth > 0 {
        let c: char = any.parse_next(parser_input)?;
        out.push(c);
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            '"' | '\'' => collect_string(parser_input, c, out)?,
            '`' => collect_template_literal(parser_input, out)?,
            _ => {}
        }
    }
    Ok(())
}

fn collect_line_comment(parser_input: &mut ParserInput, out: &mut String) -> ParseResult<()> {
    let slash: char = any.parse_next(parser_input)?;
    out.push(slash); // second /
    loop {
        let c: char = any.parse_next(parser_input)?;
        out.push(c);
        if c == '\n' {
            return Ok(());
        }
    }
}

fn collect_block_comment(parser_input: &mut ParserInput, out: &mut String) -> ParseResult<()> {
    let star: char = any.parse_next(parser_input)?;
    out.push(star); // *
    loop {
        let c: char = any.parse_next(parser_input)?;
        out.push(c);
        if c == '*' {
            let next: ParseResult<char> = peek(any).parse_next(parser_input);
                let next = next.ok();
            if next == Some('/') {
                let slash: char = any.parse_next(parser_input)?;
                out.push(slash);
                return Ok(());
            }
        }
    }
}

fn swc_parse_expression(source: &str, ts: bool, offset: u32) -> ParseResult<Box<swc::Expr>> {
    let syntax = if ts {
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        })
    } else {
        Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
        })
    };

    let input = StringInput::new(source, BytePos(offset), BytePos(offset + source.len() as u32));
    let mut parser = swc_ecma_parser::Parser::new(syntax, input, None);

    parser.parse_expr().map_err(|e| {
        e.into_diagnostic(&swc_common::errors::Handler::with_emitter(
            true,
            false,
            Box::new(swc_common::errors::EmitterWriter::new(
                Box::new(std::io::sink()),
                None,
                false,
                false,
            )),
        ))
        .cancel();
        winnow::error::ContextError::new()
    })
}
