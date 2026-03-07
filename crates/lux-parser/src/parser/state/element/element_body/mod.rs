use lux_ast::template::root::Fragment;
use winnow::Result;
use winnow::combinator::opt;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::state::fragment::parse_fragment_until;
use crate::parser::utils::helpers::skip_whitespace;

mod close_tag;
mod content;
mod this_expression;

use close_tag::{eat_closing_tag, maybe_eat_matching_closing_tag};
use content::{read_raw_text_content, read_textarea_content};
pub use this_expression::extract_this_expression;

pub fn parse_element_body<'a>(input: &mut Input<'a>, name: &str) -> Result<(Fragment<'a>, usize)> {
    skip_whitespace(input);

    let self_closing = opt(literal("/>")).parse_next(input)?.is_some();
    if !self_closing {
        let has_open_close = opt(literal(">")).parse_next(input)?.is_some();
        if !has_open_close {
            if input.state.loose {
                return Ok((
                    Fragment {
                        nodes: Vec::new(),
                        transparent: true,
                        dynamic: false,
                    },
                    input.current_token_start(),
                ));
            }
            literal(">").parse_next(input)?;
        }
    }

    let fragment = if self_closing || lux_utils::elements::is_void(name) {
        Fragment {
            nodes: Vec::new(),
            transparent: true,
            dynamic: false,
        }
    } else if name == "textarea" {
        let nodes = read_textarea_content(input)?;
        eat_closing_tag(input)?;
        Fragment {
            nodes,
            transparent: true,
            dynamic: false,
        }
    } else if name == "script" || name == "style" {
        let nodes = read_raw_text_content(input, name)?;
        eat_closing_tag(input)?;
        Fragment {
            nodes,
            transparent: true,
            dynamic: false,
        }
    } else {
        let fragment = parse_fragment_until(input, name)?;
        maybe_eat_matching_closing_tag(input, name)?;
        fragment
    };

    let end = input.previous_token_end();
    Ok((fragment, end))
}
