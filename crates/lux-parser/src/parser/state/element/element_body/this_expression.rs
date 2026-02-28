use lux_ast::template::attribute::{AttributeNode, AttributeValue};
use lux_ast::template::tag::TextOrExpressionTag;
use oxc_ast::ast::Expression;
use winnow::Result;
use winnow::error::ContextError;

pub fn extract_this_expression<'a>(
    attributes: &mut Vec<AttributeNode<'a>>,
) -> Result<Expression<'a>> {
    let this_idx = attributes.iter().position(|attribute| match attribute {
        AttributeNode::Attribute(attr) => attr.name == "this",
        _ => false,
    });

    if let Some(idx) = this_idx {
        let attr = attributes.remove(idx);
        match attr {
            AttributeNode::Attribute(a) => extract_expression_from_attr_value(a.value),
            _ => Err(ContextError::new()),
        }
    } else {
        Err(ContextError::new())
    }
}

fn extract_expression_from_attr_value(value: AttributeValue<'_>) -> Result<Expression<'_>> {
    match value {
        AttributeValue::ExpressionTag(et) => Ok(et.expression),
        AttributeValue::Sequence(mut seq) => {
            if seq.len() == 1 {
                match seq.remove(0) {
                    TextOrExpressionTag::ExpressionTag(et) => Ok(et.expression),
                    _ => Err(ContextError::new()),
                }
            } else {
                Err(ContextError::new())
            }
        }
        AttributeValue::True => Err(ContextError::new()),
    }
}
