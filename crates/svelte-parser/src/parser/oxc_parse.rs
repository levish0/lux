use oxc_allocator::Allocator;
use oxc_ast::ast::CommentKind;
use oxc_estree::{CompactJSSerializer, ESTree};
use oxc_parser::{ParseOptions, Parser, ParserReturn};
use oxc_span::SourceType;
use serde_json::Value;
use svelte_ast::span::Span;
use svelte_ast::text::{JsComment, JsCommentKind};
use svelte_ast::utils::estree::{adjust_offsets, strip_oxc_extras};
use svelte_ast::JsNode;
use winnow::Result as ParseResult;

fn make_source_type(ts: bool) -> SourceType {
    if ts {
        SourceType::tsx()
    } else {
        SourceType::mjs().with_jsx(true)
    }
}

fn make_parse_options() -> ParseOptions {
    ParseOptions {
        preserve_parens: false,
        ..Default::default()
    }
}

/// Serialize an OXC AST node implementing ESTree to serde_json::Value.
fn estree_to_value<T: ESTree>(node: &T) -> Result<Value, winnow::error::ContextError> {
    let mut serializer = CompactJSSerializer::new(false);
    node.serialize(&mut serializer);
    let json_str = serializer.into_string();
    let mut value: Value =
        serde_json::from_str(&json_str).map_err(|_| winnow::error::ContextError::new())?;
    strip_oxc_extras(&mut value);
    Ok(value)
}

/// Parse a JavaScript/TypeScript expression from source text.
/// `offset` is the byte position of the source text within the original input.
pub fn parse_expression(source: &str, ts: bool, offset: u32) -> ParseResult<JsNode> {
    let leading_ws = source.len() - source.trim_start().len();
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(winnow::error::ContextError::new());
    }
    let actual_offset = offset + leading_ws as u32;

    // Wrap in parens to disambiguate object literals from blocks,
    // then parse as a full program/script.
    let wrapper = format!("({})", trimmed);

    let allocator = Allocator::default();
    let source_type = make_source_type(ts);

    let ret = Parser::new(&allocator, &wrapper, source_type)
        .with_options(make_parse_options())
        .parse();

    if ret.panicked || !ret.errors.is_empty() {
        return Err(winnow::error::ContextError::new());
    }

    // Serialize the program to ESTree JSON and extract expression from body[0].expression
    let program_val = estree_to_value(&ret.program)?;

    let mut expr = program_val
        .get("body")
        .and_then(|b| b.as_array())
        .and_then(|arr| arr.first())
        .and_then(|stmt| stmt.get("expression"))
        .cloned()
        .ok_or_else(|| winnow::error::ContextError::new())?;

    // Positions in wrapper are 1-based (after the opening paren).
    // We want them at actual_offset, so adjust by actual_offset - 1.
    let adjustment = if actual_offset > 0 { actual_offset - 1 } else { 0 };
    adjust_offsets(&mut expr, adjustment);

    Ok(JsNode(expr))
}

/// Parse a JavaScript/TypeScript expression and collect leading comments.
/// Returns the expression and any leading comments found before the expression.
pub fn parse_expression_with_comments(
    source: &str,
    ts: bool,
    offset: u32,
) -> ParseResult<(JsNode, Vec<JsComment>)> {
    let leading_ws = source.len() - source.trim_start().len();
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(winnow::error::ContextError::new());
    }
    let actual_offset = offset + leading_ws as u32;

    let wrapper = format!("({})", trimmed);

    let allocator = Allocator::default();
    let source_type = make_source_type(ts);

    let ret = Parser::new(&allocator, &wrapper, source_type)
        .with_options(make_parse_options())
        .parse();

    if ret.panicked || !ret.errors.is_empty() {
        return Err(winnow::error::ContextError::new());
    }

    // Extract comments from program.comments
    let js_comments = collect_comments(&ret, &wrapper, actual_offset, 1);

    // Serialize and extract expression
    let program_val = estree_to_value(&ret.program)?;

    let mut expr = program_val
        .get("body")
        .and_then(|b| b.as_array())
        .and_then(|arr| arr.first())
        .and_then(|stmt| stmt.get("expression"))
        .cloned()
        .ok_or_else(|| winnow::error::ContextError::new())?;

    let adjustment = if actual_offset > 0 { actual_offset - 1 } else { 0 };
    adjust_offsets(&mut expr, adjustment);

    Ok((JsNode(expr), js_comments))
}

/// Parse a destructuring pattern from source text.
/// Wraps as arrow function params so OXC correctly handles destructuring with defaults.
pub fn parse_pattern(source: &str, ts: bool, offset: u32) -> ParseResult<JsNode> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(winnow::error::ContextError::new());
    }
    let leading_ws = source.len() - source.trim_start().len();
    let actual_offset = offset + leading_ws as u32;

    // Wrap as arrow function params: (pattern) => {}
    let wrapper = format!("({}) => {{}}", trimmed);

    let allocator = Allocator::default();
    let source_type = make_source_type(ts);

    let ret = Parser::new(&allocator, &wrapper, source_type)
        .with_options(make_parse_options())
        .parse();

    if ret.panicked || !ret.errors.is_empty() {
        return Err(winnow::error::ContextError::new());
    }

    // Serialize program and extract first param from ArrowFunctionExpression
    let program_val = estree_to_value(&ret.program)?;

    let mut param = program_val
        .get("body")
        .and_then(|b| b.as_array())
        .and_then(|arr| arr.first())
        .and_then(|stmt| stmt.get("expression"))
        .and_then(|expr| expr.get("params"))
        .and_then(|p| p.as_array())
        .and_then(|arr| arr.first())
        .cloned()
        .ok_or_else(|| winnow::error::ContextError::new())?;

    // Pattern starts at position 1 in wrapper (after opening paren).
    // We want it at actual_offset.
    let adjustment = if actual_offset > 0 { actual_offset - 1 } else { 0 };
    adjust_offsets(&mut param, adjustment);

    Ok(JsNode(param))
}

/// Parse a variable declaration from source text (for @const).
pub fn parse_var_decl(source: &str, ts: bool, offset: u32) -> ParseResult<JsNode> {
    let leading_ws = source.len() - source.trim_start().len();
    let trimmed = source.trim();

    let (full, prefix_len) = if trimmed.starts_with("const ")
        || trimmed.starts_with("let ")
        || trimmed.starts_with("var ")
    {
        (trimmed.to_string(), 0u32)
    } else {
        (format!("const {}", trimmed), 6u32)
    };

    let allocator = Allocator::default();
    let source_type = make_source_type(ts);

    let ret = Parser::new(&allocator, &full, source_type)
        .with_options(make_parse_options())
        .parse();

    if ret.panicked || !ret.errors.is_empty() {
        return Err(winnow::error::ContextError::new());
    }

    let program_val = estree_to_value(&ret.program)?;

    // Find the first statement which should be a VariableDeclaration
    let mut decl = program_val
        .get("body")
        .and_then(|b| b.as_array())
        .and_then(|arr| arr.first())
        .cloned()
        .ok_or_else(|| winnow::error::ContextError::new())?;

    // Verify it's a VariableDeclaration
    if decl.get("type").and_then(|t| t.as_str()) != Some("VariableDeclaration") {
        return Err(winnow::error::ContextError::new());
    }

    // Adjust offsets: account for leading whitespace and any prepended prefix
    let actual_offset = offset + leading_ws as u32;
    let adjustment = actual_offset.saturating_sub(prefix_len);
    adjust_offsets(&mut decl, adjustment);

    // Strip loc from VariableDeclaration and VariableDeclarator nodes.
    // Svelte reference only has loc on leaf expression nodes, not on declaration wrappers.
    if let Value::Object(ref mut obj) = decl {
        obj.remove("loc");
        if let Some(Value::Array(decls)) = obj.get_mut("declarations") {
            for d in decls.iter_mut() {
                if let Value::Object(dobj) = d {
                    dobj.remove("loc");
                }
            }
        }
    }

    Ok(JsNode(decl))
}

/// Parse a parameter list from source text (for snippet params).
/// `offset` is the position of the `(` in the overall source document.
pub fn parse_param_list(source: &str, ts: bool, offset: u32) -> ParseResult<JsNode> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(JsNode(Value::Array(vec![])));
    }

    // Wrap as (content) => {} and parse
    let wrapper = format!("({}) => {{}}", source);

    let allocator = Allocator::default();
    let source_type = make_source_type(ts);

    let ret = Parser::new(&allocator, &wrapper, source_type)
        .with_options(make_parse_options())
        .parse();

    if ret.panicked || !ret.errors.is_empty() {
        return Ok(JsNode(Value::Array(vec![])));
    }

    let program_val = estree_to_value(&ret.program)?;

    let mut params = program_val
        .get("body")
        .and_then(|b| b.as_array())
        .and_then(|arr| arr.first())
        .and_then(|stmt| stmt.get("expression"))
        .and_then(|expr| expr.get("params"))
        .cloned()
        .unwrap_or(Value::Array(vec![]));

    // The wrapper has `(` at position 0, matching the offset of the `(` in the source
    adjust_offsets(&mut params, offset);

    Ok(JsNode(params))
}

/// Parse a full program (for <script> content).
/// Returns the Program JsNode and collected comments.
pub fn parse_program(
    source: &str,
    ts: bool,
    offset: u32,
) -> ParseResult<(JsNode, Vec<JsComment>)> {
    let allocator = Allocator::default();
    let source_type = make_source_type(ts);

    let ret = Parser::new(&allocator, source, source_type)
        .with_options(make_parse_options())
        .parse();

    if ret.panicked {
        return Err(winnow::error::ContextError::new());
    }

    // Serialize the Program using ESTree serializer
    let mut value = estree_to_value(&ret.program)?;
    adjust_offsets(&mut value, offset);

    // Strip leadingComments/trailingComments from Program node.
    // Svelte attaches comments to individual statements, not to the Program.
    if let Value::Object(ref mut obj) = value {
        obj.remove("leadingComments");
        obj.remove("trailingComments");
    }

    // Collect comments from program.comments
    let js_comments = collect_comments(&ret, source, offset, 0);

    Ok((JsNode(value), js_comments))
}

/// Collect comments from parser's program.comments.
/// `wrapper_offset` is the number of bytes prepended as wrapper before the actual source.
fn collect_comments(
    ret: &ParserReturn,
    source: &str,
    target_offset: u32,
    wrapper_offset: u32,
) -> Vec<JsComment> {
    let mut js_comments = Vec::new();
    for comment in ret.program.comments.iter() {
        let span_start = comment.span.start as usize;
        let span_end = comment.span.end as usize;
        if span_end > source.len() {
            continue;
        }

        // Compute the adjusted positions
        let c_start = (comment.span.start + target_offset - wrapper_offset) as usize;
        let c_end = (comment.span.end + target_offset - wrapper_offset) as usize;

        // Extract comment text from source
        let comment_text = &source[span_start..span_end];
        let (kind, content) = if comment_text.starts_with("//") {
            (JsCommentKind::Line, comment_text[2..].to_string())
        } else if comment_text.starts_with("/*") && comment_text.ends_with("*/") {
            (
                JsCommentKind::Block,
                comment_text[2..comment_text.len() - 2].to_string(),
            )
        } else {
            // Fallback: use OXC's CommentKind
            match comment.kind {
                CommentKind::Line => (JsCommentKind::Line, comment_text.to_string()),
                CommentKind::SingleLineBlock | CommentKind::MultiLineBlock => {
                    (JsCommentKind::Block, comment_text.to_string())
                }
            }
        };

        js_comments.push(JsComment {
            span: Some(Span::new(c_start, c_end)),
            kind,
            value: content,
        });
    }
    js_comments.sort_by_key(|c| c.span.map_or(0, |s| s.start));
    js_comments
}

/// Validate that a JsNode represents a CallExpression or an optional chain call.
pub fn is_call_expression(node: &JsNode) -> bool {
    match node.0.get("type").and_then(|t| t.as_str()) {
        Some("CallExpression") => true,
        Some("ChainExpression") => {
            // Check if the inner expression is a CallExpression
            node.0
                .get("expression")
                .and_then(|e| e.get("type"))
                .and_then(|t| t.as_str())
                == Some("CallExpression")
        }
        _ => false,
    }
}

/// Get the number of declarations in a VariableDeclaration JsNode.
pub fn var_decl_count(node: &JsNode) -> usize {
    node.0
        .get("declarations")
        .and_then(|d| d.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0)
}
