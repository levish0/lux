use lux_ast::template::root::ScriptContext;
use winnow::Result;
use winnow::prelude::*;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::read::script::read_script;
use crate::parser::read::style::read_style;
use crate::parser::state::element::attribute::static_attr::read_static_attributes;
use crate::parser::utils::helpers::skip_whitespace;

pub fn parse_script_or_style<'a>(
    input: &mut Input<'a>,
    start: usize,
    name: &'a str,
) -> Result<()> {
    let attributes = read_static_attributes(input)?;

    skip_whitespace(input);
    literal(">").parse_next(input)?;

    match name {
        "script" => {
            let script = read_script(input, start, attributes)?;
            match script.context {
                ScriptContext::Module => {
                    input.state.module = Some(script);
                }
                ScriptContext::Default => {
                    input.state.instance = Some(script);
                }
            }
        }
        "style" => {
            let style = read_style(input, start, attributes)?;
            input.state.css = Some(style);
        }
        _ => {}
    }

    Ok(())
}
