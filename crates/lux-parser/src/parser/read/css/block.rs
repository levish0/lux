use lux_ast::css::stylesheet::{CssBlock, CssBlockChild, CssDeclaration};
use winnow::Result;
use winnow::error::ContextError;

use super::rule::{parse_css_atrule, parse_css_rule};
use super::value::CssParser;

pub fn parse_css_block<'a>(p: &mut CssParser<'a>) -> Result<CssBlock<'a>> {
    let start = p.index;
    p.eat_required("{")?;

    let mut children: Vec<CssBlockChild<'a>> = Vec::new();

    while !p.at_end() {
        p.skip_ws_and_comments();

        if p.matches("}") {
            break;
        }

        children.push(parse_block_item(p)?);
    }

    p.eat_required("}")?;

    Ok(CssBlock {
        span: p.span(start, p.index),
        children,
    })
}

fn parse_block_item<'a>(p: &mut CssParser<'a>) -> Result<CssBlockChild<'a>> {
    if p.matches("@") {
        return Ok(CssBlockChild::Atrule(parse_css_atrule(p)?));
    }

    // Look ahead: read a value, check if next char is '{' (nested rule) or not (declaration)
    let saved = p.index;
    let _ = p.read_css_value();
    let is_rule = p.matches("{");
    p.index = saved;

    if is_rule {
        Ok(CssBlockChild::Rule(parse_css_rule(p)?))
    } else {
        Ok(CssBlockChild::Declaration(parse_declaration(p)?))
    }
}

fn parse_declaration<'a>(p: &mut CssParser<'a>) -> Result<CssDeclaration<'a>> {
    let start = p.index;

    // Read property name (until whitespace or colon)
    let prop_start = p.index;
    while !p.at_end() {
        let b = p.source.as_bytes()[p.index];
        if b.is_ascii_whitespace() || b == b':' {
            break;
        }
        p.index += 1;
    }
    let property = &p.source[prop_start..p.index];

    p.skip_whitespace();
    p.eat_required(":")?;
    p.skip_whitespace();

    let value = p.read_css_value();

    if value.is_empty() && !property.starts_with("--") {
        return Err(ContextError::new());
    }

    let end = p.index;

    if !p.matches("}") {
        p.eat_required(";")?;
    }

    Ok(CssDeclaration {
        span: p.span(start, end),
        property,
        value,
    })
}
