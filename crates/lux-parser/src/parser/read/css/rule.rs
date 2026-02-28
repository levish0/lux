use lux_ast::css::stylesheet::{CssAtrule, CssRule};
use winnow::Result;

use super::block::parse_css_block;
use super::parser::CssParser;
use super::selector::parse_selector_list;

pub fn parse_css_rule<'a>(p: &mut CssParser<'a>) -> Result<CssRule<'a>> {
    let start = p.index;
    let prelude = parse_selector_list(p, false)?;
    let block = parse_css_block(p)?;

    Ok(CssRule {
        span: p.span(start, p.index),
        prelude,
        block,
    })
}

pub fn parse_css_atrule<'a>(p: &mut CssParser<'a>) -> Result<CssAtrule<'a>> {
    let start = p.index;
    p.eat_required("@")?;

    let name = p.read_identifier()?;
    let prelude = p.read_css_value();

    let block = if p.matches("{") {
        Some(parse_css_block(p)?)
    } else {
        p.eat_required(";")?;
        None
    };

    Ok(CssAtrule {
        span: p.span(start, p.index),
        name,
        prelude,
        block,
    })
}
