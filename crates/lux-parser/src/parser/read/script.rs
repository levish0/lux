use lux_ast::common::Span;
use lux_ast::template::attribute::{Attribute, AttributeValue};
use lux_ast::template::root::{Script, ScriptContext};
use lux_ast::template::tag::TextOrExpressionTag;
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::{literal, take_until};

use crate::input::Input;
use crate::parser::utils::helpers::skip_whitespace;

pub fn read_script<'a>(
    input: &mut Input<'a>,
    start: usize,
    attributes: Vec<Attribute<'a>>,
) -> Result<Script<'a>> {
    let content_start = input.current_token_start();

    let data: &str = take_until(0.., "</script").parse_next(input)?;

    literal("</script").parse_next(input)?;
    skip_whitespace(input);
    literal(">").parse_next(input)?;

    let end = input.previous_token_end();

    let context = detect_script_context(&attributes);

    let allocator = input.state.allocator;
    let template = input.state.template;
    let ts = input.state.ts;

    // Padding trick (matches Svelte's read_script logic):
    // source = template[0..script_start].replace(/[^\n]/g, ' ') + data
    // This ensures OXC produces spans that align with the original template positions.
    let padding = &template[..content_start];
    let padded: String = padding
        .chars()
        .map(|c| if c == '\n' { '\n' } else { ' ' })
        .collect::<String>()
        + data;

    // Allocate padded source in the arena so the Program AST can reference it.
    let padded_ref: &'a str = allocator.alloc_str(&padded);

    let source_type = if ts {
        SourceType::ts().with_module(true)
    } else {
        SourceType::mjs()
    };

    let parse_result = OxcParser::new(allocator, padded_ref, source_type).parse();

    let content = parse_result.program;

    Ok(Script {
        span: Span::new(start as u32, end as u32),
        context,
        content,
        attributes,
    })
}

fn detect_script_context(attributes: &[Attribute<'_>]) -> ScriptContext {
    for attr in attributes {
        // <script module> — boolean attribute
        if attr.name == "module" {
            return ScriptContext::Module;
        }
        // <script context="module"> — legacy syntax
        if attr.name == "context" {
            if let AttributeValue::Sequence(seq) = &attr.value {
                if seq.len() == 1 {
                    if let Some(TextOrExpressionTag::Text(text)) = seq.first() {
                        if text.data == "module" {
                            return ScriptContext::Module;
                        }
                    }
                }
            }
        }
    }
    ScriptContext::Default
}
