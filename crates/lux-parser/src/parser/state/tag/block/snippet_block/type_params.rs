use winnow::Result;
use winnow::prelude::*;
use winnow::stream::Location as StreamLocation;
use winnow::token::any;

use crate::input::Input;
use crate::parser::utils::scanner::find_matching_bracket;

pub(super) fn read_type_params<'a>(input: &mut Input<'a>) -> Result<Option<&'a str>> {
    let template = input.state.template;
    let pos = input.current_token_start();

    if let Some(end) = find_matching_bracket(template, pos + 1, '<') {
        let params = &template[pos..=end];

        // Advance input past the angle bracket range.
        let advance = end + 1 - pos;
        for _ in 0..advance {
            let _: char = any.parse_next(input)?;
        }

        Ok(Some(params))
    } else {
        Ok(None)
    }
}
