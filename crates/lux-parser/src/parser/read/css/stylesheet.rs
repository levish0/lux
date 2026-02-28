use lux_ast::css::stylesheet::StyleSheetChild;
use winnow::Result;

use super::parser::CssParser;
use super::rule::{parse_css_atrule, parse_css_rule};

pub fn parse_stylesheet(source: &str, offset: u32) -> Result<Vec<StyleSheetChild<'_>>> {
    let mut parser = CssParser::new(source, offset);
    let mut children = Vec::new();

    loop {
        parser.skip_ws_and_comments();

        if parser.at_end() || parser.matches("</style") {
            break;
        }

        if parser.matches("@") {
            children.push(StyleSheetChild::Atrule(parse_css_atrule(&mut parser)?));
        } else {
            children.push(StyleSheetChild::Rule(parse_css_rule(&mut parser)?));
        }
    }

    Ok(children)
}
