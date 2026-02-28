use lux_ast::common::Span;
use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, AssignmentTarget, AssignmentTargetMaybeDefault,
    AssignmentTargetProperty, BindingPattern, Expression, ObjectExpression, ObjectPropertyKind,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct CollectedBinding<'a> {
    pub name: &'a str,
    pub span: Span,
}

pub(super) fn collect_pattern_bindings<'a>(pattern: &'a BindingPattern<'a>) -> Vec<CollectedBinding<'a>> {
    pattern
        .get_binding_identifiers()
        .into_iter()
        .map(|identifier| CollectedBinding {
            name: identifier.name.as_str(),
            span: identifier.span,
        })
        .collect()
}

pub(super) fn collect_destructuring_expression_bindings<'a>(
    expression: &'a Expression<'a>,
) -> Vec<CollectedBinding<'a>> {
    let mut bindings = Vec::new();
    collect_expression_bindings(expression, &mut bindings);
    bindings
}

fn collect_expression_bindings<'a>(
    expression: &'a Expression<'a>,
    bindings: &mut Vec<CollectedBinding<'a>>,
) {
    match expression {
        Expression::Identifier(identifier) => {
            push_binding(identifier.name.as_str(), identifier.span, bindings);
        }
        Expression::ArrayExpression(array_expression) => {
            collect_array_expression_bindings(array_expression, bindings);
        }
        Expression::ObjectExpression(object_expression) => {
            collect_object_expression_bindings(object_expression, bindings);
        }
        Expression::AssignmentExpression(assignment_expression) => {
            collect_assignment_target_bindings(&assignment_expression.left, bindings);
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            collect_expression_bindings(&parenthesized.expression, bindings);
        }
        _ => {}
    }
}

fn collect_array_expression_bindings<'a>(
    array_expression: &'a ArrayExpression<'a>,
    bindings: &mut Vec<CollectedBinding<'a>>,
) {
    for element in &array_expression.elements {
        collect_array_element_bindings(element, bindings);
    }
}

fn collect_array_element_bindings<'a>(
    element: &'a ArrayExpressionElement<'a>,
    bindings: &mut Vec<CollectedBinding<'a>>,
) {
    match element {
        ArrayExpressionElement::SpreadElement(spread) => {
            collect_expression_bindings(&spread.argument, bindings);
        }
        ArrayExpressionElement::Identifier(identifier) => {
            push_binding(identifier.name.as_str(), identifier.span, bindings);
        }
        ArrayExpressionElement::ArrayExpression(array_expression) => {
            collect_array_expression_bindings(array_expression, bindings);
        }
        ArrayExpressionElement::ObjectExpression(object_expression) => {
            collect_object_expression_bindings(object_expression, bindings);
        }
        ArrayExpressionElement::AssignmentExpression(assignment_expression) => {
            collect_assignment_target_bindings(&assignment_expression.left, bindings);
        }
        ArrayExpressionElement::ParenthesizedExpression(parenthesized) => {
            collect_expression_bindings(&parenthesized.expression, bindings);
        }
        _ => {}
    }
}

fn collect_object_expression_bindings<'a>(
    object_expression: &'a ObjectExpression<'a>,
    bindings: &mut Vec<CollectedBinding<'a>>,
) {
    for property in &object_expression.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                collect_expression_bindings(&property.value, bindings);
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                collect_expression_bindings(&spread.argument, bindings);
            }
        }
    }
}

fn collect_assignment_target_bindings<'a>(
    assignment_target: &'a AssignmentTarget<'a>,
    bindings: &mut Vec<CollectedBinding<'a>>,
) {
    match assignment_target {
        AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            push_binding(identifier.name.as_str(), identifier.span, bindings);
        }
        AssignmentTarget::ArrayAssignmentTarget(array_target) => {
            for element in array_target.elements.iter().flatten() {
                collect_assignment_target_maybe_default_bindings(element, bindings);
            }
            if let Some(rest) = &array_target.rest {
                collect_assignment_target_bindings(&rest.target, bindings);
            }
        }
        AssignmentTarget::ObjectAssignmentTarget(object_target) => {
            for property in &object_target.properties {
                match property {
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(identifier) => {
                        push_binding(identifier.binding.name.as_str(), identifier.binding.span, bindings);
                    }
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                        collect_assignment_target_maybe_default_bindings(&property.binding, bindings);
                    }
                }
            }
            if let Some(rest) = &object_target.rest {
                collect_assignment_target_bindings(&rest.target, bindings);
            }
        }
        _ => {}
    }
}

fn collect_assignment_target_maybe_default_bindings<'a>(
    assignment_target: &'a AssignmentTargetMaybeDefault<'a>,
    bindings: &mut Vec<CollectedBinding<'a>>,
) {
    match assignment_target {
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
            collect_assignment_target_bindings(&with_default.binding, bindings);
        }
        AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(identifier) => {
            push_binding(identifier.name.as_str(), identifier.span, bindings);
        }
        AssignmentTargetMaybeDefault::ArrayAssignmentTarget(array_target) => {
            for element in array_target.elements.iter().flatten() {
                collect_assignment_target_maybe_default_bindings(element, bindings);
            }
            if let Some(rest) = &array_target.rest {
                collect_assignment_target_bindings(&rest.target, bindings);
            }
        }
        AssignmentTargetMaybeDefault::ObjectAssignmentTarget(object_target) => {
            for property in &object_target.properties {
                match property {
                    AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(identifier) => {
                        push_binding(identifier.binding.name.as_str(), identifier.binding.span, bindings);
                    }
                    AssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                        collect_assignment_target_maybe_default_bindings(&property.binding, bindings);
                    }
                }
            }
            if let Some(rest) = &object_target.rest {
                collect_assignment_target_bindings(&rest.target, bindings);
            }
        }
        _ => {}
    }
}

fn push_binding<'a>(name: &'a str, span: Span, bindings: &mut Vec<CollectedBinding<'a>>) {
    bindings.push(CollectedBinding { name, span });
}
