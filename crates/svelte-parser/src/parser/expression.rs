use swc_ecma_ast as swc;
use winnow::Result as ParseResult;
use winnow::stream::Location;

use super::ParserInput;
use super::bracket::scan_expression_content;
use super::swc_parse::parse_expression;

/// Read a JS/TS expression enclosed in `{ ... }`.
/// Consumes from `{` to matching `}`, parses with SWC.
pub fn read_expression(parser_input: &mut ParserInput) -> ParseResult<Box<swc::Expr>> {
    let start = parser_input.current_token_start();
    let content = scan_expression_content(parser_input)?;
    let ts = parser_input.state.ts;
    // offset is start + 1 (past the opening {)
    parse_expression(content, ts, (start + 1) as u32)
}
