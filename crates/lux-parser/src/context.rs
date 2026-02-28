use winnow::Result;
use winnow::error::ContextError;

use crate::input::Input;

const MAX_RECURSION_DEPTH: u32 = 256;

/// Run a parser while tracking template recursion depth.
///
/// This mirrors the structure used in sevenmark_parser: depth is always
/// increased/decreased in one place so nested fragment parsing is consistent.
pub fn with_depth<'a, T, F>(input: &mut Input<'a>, parser: F) -> Result<T>
where
    F: FnOnce(&mut Input<'a>) -> Result<T>,
{
    if input.state.depth >= MAX_RECURSION_DEPTH {
        return Err(ContextError::new());
    }

    input.state.depth += 1;
    let result = parser(input);
    input.state.depth -= 1;

    result
}

pub fn is_top_level(input: &Input<'_>) -> bool {
    input.state.depth == 0
}
