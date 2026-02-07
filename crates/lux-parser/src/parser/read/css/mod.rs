mod block;
mod rule;
mod selector;
pub mod value;

use lux_ast::css::stylesheet::StyleSheetChild;
use winnow::Result;

use self::rule::{parse_css_atrule, parse_css_rule};
use self::value::CssParser;

pub fn parse_stylesheet<'a>(source: &'a str, offset: u32) -> Result<Vec<StyleSheetChild<'a>>> {
    let mut p = CssParser::new(source, offset);
    let mut children = Vec::new();

    loop {
        p.skip_ws_and_comments();

        if p.at_end() || p.matches("</style") {
            break;
        }

        if p.matches("@") {
            children.push(StyleSheetChild::Atrule(parse_css_atrule(&mut p)?));
        } else {
            children.push(StyleSheetChild::Rule(parse_css_rule(&mut p)?));
        }
    }

    Ok(children)
}
