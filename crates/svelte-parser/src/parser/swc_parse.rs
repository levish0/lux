use svelte_ast::span::Span;
use svelte_ast::text::{JsComment, JsCommentKind};
use swc_common::BytePos;
use swc_common::comments::{CommentKind, SingleThreadedComments};
use swc_common::input::StringInput;
use swc_ecma_ast as swc;
use swc_ecma_parser::{EsSyntax, Syntax, TsSyntax};
use winnow::Result as ParseResult;

fn make_syntax(ts: bool) -> Syntax {
    if ts {
        Syntax::Typescript(TsSyntax {
            tsx: true,
            ..Default::default()
        })
    } else {
        Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
        })
    }
}

fn swc_error_to_parse_error(e: swc_ecma_parser::error::Error) -> winnow::error::ContextError {
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
}

/// Parse a JavaScript/TypeScript expression from source text.
/// `offset` is the byte position of the source text within the original input.
pub fn parse_expression(source: &str, ts: bool, offset: u32) -> ParseResult<Box<swc::Expr>> {
    let leading_ws = source.len() - source.trim_start().len();
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(winnow::error::ContextError::new());
    }
    let actual_offset = offset + leading_ws as u32;

    let input = StringInput::new(
        trimmed,
        BytePos(actual_offset),
        BytePos(actual_offset + trimmed.len() as u32),
    );
    let mut parser = swc_ecma_parser::Parser::new(make_syntax(ts), input, None);

    parser.parse_expr().map_err(swc_error_to_parse_error)
}

/// Parse a JavaScript/TypeScript expression and collect leading comments.
/// Returns the expression and any leading comments found before the expression.
pub fn parse_expression_with_comments(
    source: &str,
    ts: bool,
    offset: u32,
) -> ParseResult<(Box<swc::Expr>, Vec<JsComment>)> {
    let leading_ws = source.len() - source.trim_start().len();
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(winnow::error::ContextError::new());
    }
    let actual_offset = offset + leading_ws as u32;

    let comments = SingleThreadedComments::default();
    let input = StringInput::new(
        trimmed,
        BytePos(actual_offset),
        BytePos(actual_offset + trimmed.len() as u32),
    );
    let mut parser = swc_ecma_parser::Parser::new(make_syntax(ts), input, Some(&comments));

    let expr = parser.parse_expr().map_err(swc_error_to_parse_error)?;

    // Collect leading comments (comments before the expression start)
    let (leading_map, _trailing_map) = comments.borrow_all();
    let mut js_comments = Vec::new();
    for (_pos, comment_vec) in leading_map.iter() {
        for comment in comment_vec {
            let kind = match comment.kind {
                CommentKind::Line => JsCommentKind::Line,
                CommentKind::Block => JsCommentKind::Block,
            };
            js_comments.push(JsComment {
                span: Some(Span::new(
                    comment.span.lo.0 as usize,
                    comment.span.hi.0 as usize,
                )),
                kind,
                value: comment.text.to_string(),
            });
        }
    }
    js_comments.sort_by_key(|c| c.span.map_or(0, |s| s.start));

    Ok((expr, js_comments))
}

/// Parse a destructuring pattern from source text.
/// Wraps as arrow function params so SWC correctly handles destructuring with defaults.
pub fn parse_pattern(source: &str, ts: bool, offset: u32) -> ParseResult<Box<swc::Pat>> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(winnow::error::ContextError::new());
    }
    let leading_ws = source.len() - source.trim_start().len();
    let actual_offset = offset + leading_ws as u32;

    // Wrap as arrow function params so SWC parses destructuring with defaults
    let wrapper = format!("({}) => {{}}", trimmed);
    let parse_offset = if actual_offset > 0 {
        actual_offset - 1
    } else {
        0
    };

    let input = StringInput::new(
        &wrapper,
        BytePos(parse_offset),
        BytePos(parse_offset + wrapper.len() as u32),
    );
    let mut parser = swc_ecma_parser::Parser::new(make_syntax(ts), input, None);

    let expr = parser.parse_expr().map_err(swc_error_to_parse_error)?;

    match *expr {
        swc::Expr::Arrow(arrow) if !arrow.params.is_empty() => {
            Ok(Box::new(arrow.params.into_iter().next().unwrap()))
        }
        _ => {
            // Fallback: parse as expression and convert
            let expr = parse_expression(source, ts, offset)?;
            Ok(Box::new(expr_to_pat(*expr)))
        }
    }
}

/// Parse a variable declaration from source text (for @const).
pub fn parse_var_decl(source: &str, ts: bool) -> ParseResult<Box<swc::VarDecl>> {
    let full = if source.trim_start().starts_with("const ")
        || source.trim_start().starts_with("let ")
        || source.trim_start().starts_with("var ")
    {
        source.trim().to_string()
    } else {
        format!("const {}", source.trim())
    };

    let input = StringInput::new(&full, BytePos(0), BytePos(full.len() as u32));
    let mut parser = swc_ecma_parser::Parser::new(make_syntax(ts), input, None);

    let module = parser.parse_module().map_err(swc_error_to_parse_error)?;

    for item in module.body {
        if let swc::ModuleItem::Stmt(swc::Stmt::Decl(swc::Decl::Var(var_decl))) = item {
            return Ok(var_decl);
        }
    }

    Err(winnow::error::ContextError::new())
}

/// Parse a parameter list from source text (for snippet params).
/// `offset` is the position of the `(` in the overall source document.
pub fn parse_param_list(source: &str, ts: bool, offset: u32) -> ParseResult<Vec<swc::Pat>> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }

    // Wrap as (content) => {} and parse at offset so SWC positions are correct
    let wrapper = format!("({}) => {{}}", source);
    let expr = parse_expression(&wrapper, ts, offset)?;
    match *expr {
        swc::Expr::Arrow(arrow) => Ok(arrow.params),
        _ => Ok(vec![]),
    }
}

/// Convert an expression AST node to a pattern (for destructuring fallback).
pub fn expr_to_pat(expr: swc::Expr) -> swc::Pat {
    match expr {
        swc::Expr::Ident(id) => swc::Pat::Ident(swc::BindingIdent { id, type_ann: None }),
        swc::Expr::Array(arr) => swc::Pat::Array(swc::ArrayPat {
            span: arr.span,
            elems: arr
                .elems
                .into_iter()
                .map(|e| e.map(|e| expr_to_pat(*e.expr)))
                .collect(),
            optional: false,
            type_ann: None,
        }),
        swc::Expr::Object(obj) => swc::Pat::Object(swc::ObjectPat {
            span: obj.span,
            props: obj
                .props
                .into_iter()
                .filter_map(|p| match p {
                    swc::PropOrSpread::Prop(prop) => match *prop {
                        swc::Prop::Shorthand(id) => {
                            Some(swc::ObjectPatProp::Assign(swc::AssignPatProp {
                                span: id.span,
                                key: swc::BindingIdent { id, type_ann: None },
                                value: None,
                            }))
                        }
                        swc::Prop::KeyValue(kv) => {
                            Some(swc::ObjectPatProp::KeyValue(swc::KeyValuePatProp {
                                key: kv.key,
                                value: Box::new(expr_to_pat(*kv.value)),
                            }))
                        }
                        _ => None,
                    },
                    swc::PropOrSpread::Spread(s) => Some(swc::ObjectPatProp::Rest(swc::RestPat {
                        span: s.dot3_token,
                        dot3_token: s.dot3_token,
                        arg: Box::new(expr_to_pat(*s.expr)),
                        type_ann: None,
                    })),
                })
                .collect(),
            optional: false,
            type_ann: None,
        }),
        _ => swc::Pat::Expr(Box::new(expr)),
    }
}
