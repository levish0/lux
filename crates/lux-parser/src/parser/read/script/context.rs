use lux_ast::template::attribute::{Attribute, AttributeValue};
use lux_ast::template::root::ScriptContext;
use lux_ast::template::tag::TextOrExpressionTag;

use crate::error::{ErrorKind, ParseError, ParseWarning, WarningKind};
use crate::input::Input;

const RESERVED_ATTRIBUTES: &[&str] = &["server", "client", "worker", "test", "default"];
const ALLOWED_ATTRIBUTES: &[&str] = &["context", "generics", "lang", "module"];

pub(super) fn detect_script_context(
    input: &mut Input<'_>,
    attributes: &[Attribute<'_>],
) -> ScriptContext {
    let mut context = ScriptContext::Default;

    for attr in attributes {
        if RESERVED_ATTRIBUTES.contains(&attr.name) {
            input.state.errors.push(ParseError::with_code(
                ErrorKind::InvalidScript,
                "script_reserved_attribute",
                attr.span,
                format!("The `{}` attribute is reserved and cannot be used", attr.name),
            ));
        }

        if !ALLOWED_ATTRIBUTES.contains(&attr.name) {
            input.state.warnings.push(ParseWarning::new(
                WarningKind::InvalidScript,
                "script_unknown_attribute",
                attr.span,
                "Unrecognized attribute; should be one of `generics`, `lang` or `module`",
            ));
            continue;
        }

        if attr.name == "module" {
            if !matches!(attr.value, AttributeValue::True) {
                input.state.errors.push(ParseError::with_code(
                    ErrorKind::InvalidScript,
                    "script_invalid_attribute_value",
                    attr.span,
                    "If supplied, `module` must be a boolean attribute",
                ));
            } else {
                context = ScriptContext::Module;
            }
            continue;
        }

        if attr.name == "context" {
            match &attr.value {
                AttributeValue::Sequence(seq)
                    if seq.len() == 1
                        && matches!(
                            seq.first(),
                            Some(TextOrExpressionTag::Text(text)) if text.data == "module"
                        ) =>
                {
                    context = ScriptContext::Module;
                }
                _ => {
                    input.state.errors.push(ParseError::with_code(
                        ErrorKind::InvalidScript,
                        "script_invalid_context",
                        attr.span,
                        "If the context attribute is supplied, its value must be \"module\"",
                    ));
                }
            }
        }
    }

    context
}
