use lux_ast::analysis::{AnalysisDiagnosticCode, AnalysisSeverity};
use lux_ast::template::attribute::{Attribute, AttributeNode, AttributeValue};
use lux_ast::template::directive::BindDirective;
use lux_ast::template::tag::TextOrExpressionTag;
use lux_metadata::bindings::get_binding_property;
use lux_utils::elements::is_svg;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;

use crate::template::context::TemplateAnalyzerContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindDirectiveTarget<'a> {
    Regular(&'a str),
    SvelteElement,
    SvelteWindow,
    SvelteDocument,
    SvelteBody,
    Other,
}

const WINDOW_ONLY_BINDINGS: &[&str] = &[
    "innerWidth",
    "innerHeight",
    "outerWidth",
    "outerHeight",
    "scrollX",
    "scrollY",
    "online",
    "devicePixelRatio",
];

const DOCUMENT_ONLY_BINDINGS: &[&str] = &[
    "activeElement",
    "fullscreenElement",
    "pointerLockElement",
    "visibilityState",
];

pub(crate) fn validate_bind_directive_expression(
    directive: &BindDirective<'_>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    if directive.name == "group"
        && matches!(directive.expression, Expression::SequenceExpression(_))
    {
        context.add_diagnostic(
            AnalysisSeverity::Error,
            AnalysisDiagnosticCode::BindDirectiveGroupInvalidExpression,
            "bind:group does not support getter/setter sequence expressions",
            directive.expression.span(),
        );
    }

    if is_valid_bind_expression(&directive.expression) {
        return;
    }

    context.add_diagnostic(
        AnalysisSeverity::Error,
        AnalysisDiagnosticCode::BindDirectiveInvalidExpression,
        "bind directive expects an assignable expression or getter/setter pair",
        directive.expression.span(),
    );
}

pub(crate) fn validate_bind_directive_target(
    directive: &BindDirective<'_>,
    attributes: &[AttributeNode<'_>],
    target: BindDirectiveTarget<'_>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    if directive.name == "this" {
        return;
    }

    let Some(property) = get_binding_property(directive.name) else {
        if matches!(
            target,
            BindDirectiveTarget::Regular(_)
                | BindDirectiveTarget::SvelteElement
                | BindDirectiveTarget::SvelteWindow
                | BindDirectiveTarget::SvelteDocument
                | BindDirectiveTarget::SvelteBody
        ) {
            context.add_diagnostic(
                AnalysisSeverity::Error,
                AnalysisDiagnosticCode::BindDirectiveUnknownName,
                format!("Unknown bind directive name `{}`", directive.name),
                directive.span,
            );
        }

        return;
    };

    let is_window_only = WINDOW_ONLY_BINDINGS.contains(&directive.name);
    let is_document_only = DOCUMENT_ONLY_BINDINGS.contains(&directive.name);

    let is_valid_target = match target {
        BindDirectiveTarget::Regular(element_name) => {
            !is_window_only
                && !is_document_only
                && (property.valid_elements.is_empty()
                    || property.valid_elements.contains(&element_name))
        }
        BindDirectiveTarget::SvelteWindow => is_window_only,
        BindDirectiveTarget::SvelteDocument => is_document_only,
        BindDirectiveTarget::SvelteBody => {
            !is_window_only
                && !is_document_only
                && (property.valid_elements.is_empty() || property.valid_elements.contains(&"body"))
        }
        BindDirectiveTarget::SvelteElement => true,
        BindDirectiveTarget::Other => true,
    };

    if !is_valid_target {
        context.add_diagnostic(
            AnalysisSeverity::Error,
            AnalysisDiagnosticCode::BindDirectiveInvalidTarget,
            format!("bind:{} is not valid on this target", directive.name),
            directive.span,
        );
    }

    if let BindDirectiveTarget::Regular(element_name) = target {
        validate_regular_element_bind_rules(directive, attributes, element_name, context);
    }
}

fn validate_regular_element_bind_rules(
    directive: &BindDirective<'_>,
    attributes: &[AttributeNode<'_>],
    element_name: &str,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    if element_name == "input" && directive.name != "this" {
        let type_attr = find_attribute(attributes, "type");

        if let Some(type_attr) = type_attr {
            let type_text_value = static_text_attribute_value(type_attr);

            if type_text_value.is_none() {
                if directive.name != "value" || matches!(type_attr.value, AttributeValue::True) {
                    context.add_diagnostic(
                        AnalysisSeverity::Error,
                        AnalysisDiagnosticCode::BindDirectiveInputTypeInvalid,
                        "input `type` attribute must be static text for this bind directive",
                        type_attr.span,
                    );
                }
            } else {
                validate_input_type_match(directive, type_text_value, context);
            }
        } else {
            validate_input_type_match(directive, None, context);
        }
    }

    if element_name == "select" && directive.name != "this" {
        if let Some(multiple_attr) = find_attribute(attributes, "multiple") {
            if !is_text_attribute(multiple_attr)
                && !matches!(multiple_attr.value, AttributeValue::True)
            {
                context.add_diagnostic(
                    AnalysisSeverity::Error,
                    AnalysisDiagnosticCode::BindDirectiveSelectMultipleDynamic,
                    "`multiple` attribute on `<select>` must not be dynamic when using bind",
                    multiple_attr.span,
                );
            }
        }
    }

    if matches!(directive.name, "innerText" | "innerHTML" | "textContent") {
        match find_attribute(attributes, "contenteditable") {
            None => context.add_diagnostic(
                AnalysisSeverity::Error,
                AnalysisDiagnosticCode::BindDirectiveContenteditableMissing,
                "contenteditable attribute is required for this bind directive",
                directive.span,
            ),
            Some(contenteditable_attr) => {
                if !is_text_attribute(contenteditable_attr)
                    && !matches!(contenteditable_attr.value, AttributeValue::True)
                {
                    context.add_diagnostic(
                        AnalysisSeverity::Error,
                        AnalysisDiagnosticCode::BindDirectiveContenteditableDynamic,
                        "contenteditable attribute must be static when using this bind directive",
                        contenteditable_attr.span,
                    );
                }
            }
        }
    }

    if directive.name == "offsetWidth" && is_svg(element_name) {
        context.add_diagnostic(
            AnalysisSeverity::Error,
            AnalysisDiagnosticCode::BindDirectiveInvalidTarget,
            "bind:offsetWidth is not valid on SVG elements (use bind:clientWidth instead)",
            directive.span,
        );
    }
}

fn validate_input_type_match(
    directive: &BindDirective<'_>,
    type_text_value: Option<&str>,
    context: &mut TemplateAnalyzerContext<'_>,
) {
    if directive.name == "checked" && type_text_value != Some("checkbox") {
        context.add_diagnostic(
            AnalysisSeverity::Error,
            AnalysisDiagnosticCode::BindDirectiveInputTypeMismatch,
            "bind:checked requires `<input type=\"checkbox\">`",
            directive.span,
        );
    }

    if directive.name == "files" && type_text_value != Some("file") {
        context.add_diagnostic(
            AnalysisSeverity::Error,
            AnalysisDiagnosticCode::BindDirectiveInputTypeMismatch,
            "bind:files requires `<input type=\"file\">`",
            directive.span,
        );
    }
}

fn find_attribute<'a>(
    attributes: &'a [AttributeNode<'a>],
    name: &str,
) -> Option<&'a Attribute<'a>> {
    attributes.iter().find_map(|attribute_node| {
        let AttributeNode::Attribute(attribute) = attribute_node else {
            return None;
        };

        if attribute.name == name {
            Some(attribute)
        } else {
            None
        }
    })
}

fn is_text_attribute(attribute: &Attribute<'_>) -> bool {
    static_text_attribute_value(attribute).is_some()
}

fn static_text_attribute_value<'a>(attribute: &'a Attribute<'a>) -> Option<&'a str> {
    let AttributeValue::Sequence(chunks) = &attribute.value else {
        return None;
    };

    if chunks.len() != 1 {
        return None;
    }

    match &chunks[0] {
        TextOrExpressionTag::Text(text) => Some(text.data),
        TextOrExpressionTag::ExpressionTag(_) => None,
    }
}

fn is_valid_bind_expression(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::Identifier(_)
        | Expression::ComputedMemberExpression(_)
        | Expression::StaticMemberExpression(_)
        | Expression::PrivateFieldExpression(_) => true,
        Expression::SequenceExpression(sequence) => sequence.expressions.len() == 2,
        Expression::ParenthesizedExpression(parenthesized) => {
            is_valid_bind_expression(&parenthesized.expression)
        }
        Expression::TSAsExpression(expression) => is_valid_bind_expression(&expression.expression),
        Expression::TSSatisfiesExpression(expression) => {
            is_valid_bind_expression(&expression.expression)
        }
        Expression::TSNonNullExpression(expression) => {
            is_valid_bind_expression(&expression.expression)
        }
        Expression::TSTypeAssertion(expression) => is_valid_bind_expression(&expression.expression),
        _ => false,
    }
}
