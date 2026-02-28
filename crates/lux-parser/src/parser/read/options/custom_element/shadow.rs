use lux_ast::template::attribute::Attribute;
use lux_ast::template::root::CustomElementShadow;
use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::ast::Expression;

use crate::error::ParseError;

use super::errors::invalid_custom_element_shadow;

pub(super) fn parse_custom_element_shadow<'a>(
    attr: &Attribute<'a>,
    value: &Expression<'a>,
    allocator: &'a Allocator,
) -> Result<CustomElementShadow<'a>, ParseError> {
    match value.get_inner_expression() {
        Expression::StringLiteral(lit) if lit.value.as_str() == "open" => {
            Ok(CustomElementShadow::Open)
        }
        Expression::StringLiteral(lit) if lit.value.as_str() == "none" => {
            Ok(CustomElementShadow::None)
        }
        Expression::ObjectExpression(object) => Ok(CustomElementShadow::Object(
            object.as_ref().clone_in(allocator),
        )),
        _ => Err(invalid_custom_element_shadow(attr)),
    }
}
