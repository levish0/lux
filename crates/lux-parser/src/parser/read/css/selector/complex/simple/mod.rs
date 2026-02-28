mod attribute;
mod numeric;
mod pseudo;
mod type_selector;

use lux_ast::css::selector::RelativeSelector;
use winnow::Result;

use self::attribute::parse_attribute_selector;
use self::numeric::parse_numeric_selector;
use self::pseudo::{parse_pseudo_class_selector, parse_pseudo_element_selector};
use self::type_selector::{
    parse_class_selector, parse_id_selector, parse_nesting_selector, parse_type_selector,
    parse_universal_selector,
};
use super::super::super::parser::CssParser;

pub(super) fn parse_simple_selector<'a>(
    parser: &mut CssParser<'a>,
    inside_pseudo_class: bool,
    relative_selector: &mut RelativeSelector<'a>,
) -> Result<()> {
    if parse_nesting_selector(parser, relative_selector) {
        return Ok(());
    }

    if parse_universal_selector(parser, relative_selector)? {
        return Ok(());
    }

    if parse_id_selector(parser, relative_selector)? {
        return Ok(());
    }

    if parse_class_selector(parser, relative_selector)? {
        return Ok(());
    }

    if parse_pseudo_element_selector(parser, relative_selector)? {
        return Ok(());
    }

    if parse_pseudo_class_selector(parser, relative_selector)? {
        return Ok(());
    }

    if parse_attribute_selector(parser, relative_selector)? {
        return Ok(());
    }

    if parse_numeric_selector(parser, inside_pseudo_class, relative_selector) {
        return Ok(());
    }

    let _ = parse_type_selector(parser, relative_selector)?;
    Ok(())
}
