use svelte_ast::node::FragmentNode;
use svelte_ast::span::Span;
use svelte_ast::tags::{AttachTag, ConstTag, DebugTag, ExpressionTag, HtmlTag, RenderTag};
use swc_ecma_ast as swc;
use winnow::prelude::*;
use winnow::stream::Location;
use winnow::token::{any, literal, take_while};
use winnow::Result as ParseResult;

use super::ParserInput;
use super::expression::read_expression;

/// Parse `{expression}` tag.
pub fn expression_tag_parser(parser_input: &mut ParserInput) -> ParseResult<FragmentNode> {
    let start = parser_input.current_token_start();
    let expression = read_expression(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(FragmentNode::ExpressionTag(ExpressionTag {
        span: Span::new(start, end),
        expression,
    }))
}

/// Parse `{@keyword ...}` special tags.
/// Called after `{@` has been peeked but not consumed.
pub fn special_tag_parser(parser_input: &mut ParserInput) -> ParseResult<FragmentNode> {
    let start = parser_input.current_token_start();

    // Consume {@
    literal("{@").parse_next(parser_input)?;

    // Read keyword
    let keyword: &str =
        take_while(1.., |c: char| c.is_ascii_alphabetic()).parse_next(parser_input)?;

    match keyword {
        "html" => html_tag_parser(parser_input, start),
        "debug" => debug_tag_parser(parser_input, start),
        "const" => const_tag_parser(parser_input, start),
        "render" => render_tag_parser(parser_input, start),
        "attach" => attach_tag_parser(parser_input, start),
        _ => Err(winnow::error::ContextError::new()),
    }
}

/// Parse `{@html expression}`
fn html_tag_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
    let expression = read_expression_until_close(parser_input)?;
    literal("}").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(FragmentNode::HtmlTag(HtmlTag {
        span: Span::new(start, end),
        expression,
    }))
}

/// Parse `{@debug ident1, ident2, ...}`
fn debug_tag_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    let mut identifiers = Vec::new();

    // Check if we immediately hit } (bare @debug with no identifiers)
    if peek(any).parse_next(parser_input)? != '}' {
        loop {
            take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
            let name: &str =
                take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '$')
                    .parse_next(parser_input)?;
            identifiers.push(swc::Ident::new(
                name.into(),
                swc_common::DUMMY_SP,
                Default::default(),
            ));
            take_while(0.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
            if peek(any).parse_next(parser_input)? == ',' {
                any.parse_next(parser_input)?; // consume comma
            } else {
                break;
            }
        }
    }

    literal("}").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(FragmentNode::DebugTag(DebugTag {
        span: Span::new(start, end),
        identifiers,
    }))
}

/// Parse `{@const declaration}`
fn const_tag_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;

    // Read everything until } and parse as variable declaration
    let decl_text = read_until_close_brace(parser_input)?;
    literal("}").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    let declaration = swc_parse_var_decl(&decl_text, parser_input.state.ts)?;

    Ok(FragmentNode::ConstTag(ConstTag {
        span: Span::new(start, end),
        declaration,
    }))
}

/// Parse `{@render expression()}`
fn render_tag_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
    let expression = read_expression_until_close(parser_input)?;
    literal("}").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(FragmentNode::RenderTag(RenderTag {
        span: Span::new(start, end),
        expression,
    }))
}

/// Parse `{@attach expression}`
fn attach_tag_parser(parser_input: &mut ParserInput, start: usize) -> ParseResult<FragmentNode> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(parser_input)?;
    let expression = read_expression_until_close(parser_input)?;
    literal("}").parse_next(parser_input)?;
    let end = parser_input.previous_token_end();

    Ok(FragmentNode::AttachTag(AttachTag {
        span: Span::new(start, end),
        expression,
    }))
}

// --- Helpers ---

use winnow::combinator::peek;

/// Read expression text until `}` without consuming the `}`.
fn read_expression_until_close(parser_input: &mut ParserInput) -> ParseResult<Box<swc::Expr>> {
    let text = read_until_close_brace(parser_input)?;
    swc_parse_expr(&text, parser_input.state.ts)
}

/// Read text until `}` without consuming it. Handles nested braces.
fn read_until_close_brace(parser_input: &mut ParserInput) -> ParseResult<String> {
    let mut buf = String::new();
    let mut depth: u32 = 0;
    loop {
        let c = peek(any).parse_next(parser_input)?;
        if c == '}' && depth == 0 {
            break;
        }
        let c: char = any.parse_next(parser_input)?;
        buf.push(c);
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            '"' | '\'' => {
                collect_string(parser_input, c, &mut buf)?;
            }
            '`' => {
                collect_template(parser_input, &mut buf)?;
            }
            _ => {}
        }
    }
    Ok(buf)
}

fn collect_string(parser_input: &mut ParserInput, quote: char, out: &mut String) -> ParseResult<()> {
    loop {
        let c: char = any.parse_next(parser_input)?;
        out.push(c);
        if c == quote {
            return Ok(());
        }
        if c == '\\' {
            let esc: char = any.parse_next(parser_input)?;
            out.push(esc);
        }
    }
}

fn collect_template(parser_input: &mut ParserInput, out: &mut String) -> ParseResult<()> {
    loop {
        let c: char = any.parse_next(parser_input)?;
        out.push(c);
        match c {
            '`' => return Ok(()),
            '\\' => {
                let esc: char = any.parse_next(parser_input)?;
                out.push(esc);
            }
            _ => {}
        }
    }
}

fn swc_parse_expr(source: &str, ts: bool) -> ParseResult<Box<swc::Expr>> {
    use swc_common::BytePos;
    use swc_common::input::StringInput;
    use swc_ecma_parser::{EsSyntax, Syntax, TsSyntax};

    let trimmed = source.trim();
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

    let input = StringInput::new(trimmed, BytePos(0), BytePos(trimmed.len() as u32));
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

fn swc_parse_var_decl(source: &str, ts: bool) -> ParseResult<Box<swc::VarDecl>> {
    use swc_common::BytePos;
    use swc_common::input::StringInput;
    use swc_ecma_parser::{EsSyntax, Syntax, TsSyntax};

    // Wrap in "const " if not already starting with a declaration keyword
    let full = if source.trim_start().starts_with("const ")
        || source.trim_start().starts_with("let ")
        || source.trim_start().starts_with("var ")
    {
        source.trim().to_string()
    } else {
        format!("const {}", source.trim())
    };

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

    let input = StringInput::new(&full, BytePos(0), BytePos(full.len() as u32));
    let mut parser = swc_ecma_parser::Parser::new(syntax, input, None);

    // Parse as a module item (to get VarDecl)
    let module = parser.parse_module().map_err(|e| {
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
    })?;

    // Extract VarDecl from module body
    for item in module.body {
        if let swc::ModuleItem::Stmt(swc::Stmt::Decl(swc::Decl::Var(var_decl))) = item {
            return Ok(var_decl);
        }
    }

    Err(winnow::error::ContextError::new())
}
