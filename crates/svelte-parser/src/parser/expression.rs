use svelte_ast::JsNode;
use winnow::Result as ParseResult;
use winnow::stream::Location;

use super::ParserInput;
use super::bracket::scan_expression_content;
use super::oxc_parse::parse_expression;

/// Read a JS/TS expression enclosed in `{ ... }`.
/// Consumes from `{` to matching `}`, parses with OXC.
pub fn read_expression(parser_input: &mut ParserInput) -> ParseResult<JsNode> {
    let start = parser_input.current_token_start();
    let content = scan_expression_content(parser_input)?;
    let ts = parser_input.state.ts;
    let loose = parser_input.state.loose;
    // offset is start + 1 (past the opening {)
    let offset = (start + 1) as u32;
    parse_expression_or_loose(content, ts, offset, loose)
}

/// Parse an expression with loose-mode fallback.
/// In loose mode, if parsing fails, returns an empty-name Identifier
/// spanning the content's trimmed range.
pub fn parse_expression_or_loose(
    content: &str,
    ts: bool,
    offset: u32,
    loose: bool,
) -> ParseResult<JsNode> {
    match parse_expression(content, ts, offset) {
        Ok(node) => Ok(node),
        Err(_) if loose => Ok(make_empty_ident(content, offset)),
        Err(e) => Err(e),
    }
}

/// Create an empty-name Identifier covering the trimmed content span.
pub fn make_empty_ident(content: &str, offset: u32) -> JsNode {
    let leading_ws = content.len() - content.trim_start().len();
    let trimmed = content.trim();
    let start = offset + leading_ws as u32;
    let end = start + trimmed.len() as u32;
    JsNode(serde_json::json!({
        "type": "Identifier",
        "name": "",
        "start": start,
        "end": end
    }))
}
