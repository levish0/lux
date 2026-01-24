use std::sync::LazyLock;

use oxc_ast::ast::Program;
use oxc_span::SourceType;
use regex::Regex;

use svelte_ast::node::AttributeNode;
use svelte_ast::root::{Script, ScriptContext};
use svelte_ast::span::Span;
use crate::error::ErrorKind::{ElementUnclosed, JsParseError};
use crate::parser::{ParseError, Parser};

/// Regex to find `</script` (optionally with whitespace) followed by `>`.
static REGEX_CLOSING_SCRIPT_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"</script\s*>").unwrap());

/// Regex to match `</script...>` at the start of remaining input.
static REGEX_STARTS_WITH_CLOSING_SCRIPT_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^</script\s*>").unwrap());

/// Read the content of a `<script>` tag and parse it as a JS/TS program.
/// Port of reference `read/script.js`.
///
/// `start` is the byte offset of the opening `<` of `<script>`.
/// `attributes` are the already-parsed attributes of the script tag.
/// `parser.index` should be just after the `>` of the opening tag.
pub fn read_script<'a>(
    parser: &mut Parser<'a>,
    start: usize,
    attributes: Vec<AttributeNode<'a>>,
) -> Result<Script<'a>, ParseError> {
    let script_start = parser.index;

    // Read content until </script>
    let data = parser.read_until(&REGEX_CLOSING_SCRIPT_TAG);

    if parser.index >= parser.template.len() {
        if !parser.loose {
            return Err(parser.error(
                ElementUnclosed,
                parser.template.len(),
                "'<script>' was not closed".to_string(),
            ));
        }
    }

    // Consume the </script> tag
    parser.read(&REGEX_STARTS_WITH_CLOSING_SCRIPT_TAG);

    // Build padded source: everything before script_start replaced with spaces
    // (preserving newlines for correct line numbers), then the script content.
    let prefix: String = parser.template[..script_start]
        .chars()
        .map(|c| if c == '\n' { '\n' } else { ' ' })
        .collect();
    let source = format!("{}{}", prefix, data);
    let source_str = parser.allocator.alloc_str(&source);

    // Parse with OXC
    let source_type = if parser.ts {
        SourceType::ts().with_module(true)
    } else {
        SourceType::mjs()
    };

    let oxc_result = oxc_parser::Parser::new(parser.allocator, source_str, source_type).parse();

    if !oxc_result.errors.is_empty() && !parser.loose {
        let first_err = &oxc_result.errors[0];
        return Err(parser.error(
            JsParseError,
            script_start,
            format!("Script parse error: {}", first_err),
        ));
    }

    let mut program: Program<'a> = oxc_result.program;

    // Fix up program span to match script content position
    program.span = oxc_span::Span::new(script_start as u32, (script_start + data.len()) as u32);

    // Determine context from attributes
    let context = determine_script_context(&attributes);

    Ok(Script {
        span: Span::new(start, parser.index),
        context,
        content: program,
        attributes,
    })
}

/// Determine the script context (default or module) from attributes.
/// Reference checks for `module` attribute (boolean) or `context="module"`.
fn determine_script_context(attributes: &[AttributeNode]) -> ScriptContext {
    use svelte_ast::attributes::{AttributeSequenceValue, AttributeValue};

    for attr in attributes {
        if let AttributeNode::Attribute(a) = attr {
            // `<script module>` — boolean attribute
            if a.name == "module" {
                return ScriptContext::Module;
            }
            // `<script context="module">` — string attribute
            if a.name == "context" {
                if let AttributeValue::Sequence(values) = &a.value {
                    if values.len() == 1 {
                        if let Some(AttributeSequenceValue::Text(text)) = values.first() {
                            if text.data == "module" {
                                return ScriptContext::Module;
                            }
                        }
                    }
                }
            }
        }
    }
    ScriptContext::Default
}
