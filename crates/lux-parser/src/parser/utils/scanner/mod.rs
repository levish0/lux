mod skip;
mod stop;

#[cfg(test)]
mod tests;

use self::skip::{skip_block_comment, skip_line_comment, skip_string, skip_template_literal};
use self::stop::{close_for_open, should_stop_top_level};

pub(super) const EXPRESSION_NESTING_PAIRS: &[(u8, u8)] =
    &[(b'{', b'}'), (b'(', b')'), (b'[', b']')];

#[derive(Clone, Copy)]
pub(super) enum TopLevelStop<'a> {
    None,
    Expression(&'a [u8]),
    EachAs,
    AwaitClause,
}

pub fn scan_expression_boundary(source: &str, extra_stops: &[u8]) -> Option<usize> {
    scan(
        source,
        0,
        None,
        TopLevelStop::Expression(extra_stops),
        EXPRESSION_NESTING_PAIRS,
    )
}

pub fn scan_each_expression_boundary(source: &str) -> Option<usize> {
    scan(
        source,
        0,
        None,
        TopLevelStop::EachAs,
        EXPRESSION_NESTING_PAIRS,
    )
}

pub fn scan_await_expression_boundary(source: &str) -> Option<usize> {
    scan(
        source,
        0,
        None,
        TopLevelStop::AwaitClause,
        EXPRESSION_NESTING_PAIRS,
    )
}

/// Find the byte index of the closing bracket matching `open`.
///
/// `start` must point right after the opening bracket.
pub fn find_matching_bracket(source: &str, start: usize, open: char) -> Option<usize> {
    let close = match open {
        '{' => '}',
        '(' => ')',
        '[' => ']',
        '<' => '>',
        _ => return None,
    };

    if !open.is_ascii() || !close.is_ascii() {
        return None;
    }

    let pair = [(open as u8, close as u8)];
    scan(source, start, Some(close as u8), TopLevelStop::None, &pair)
}

pub(super) fn scan(
    source: &str,
    mut index: usize,
    terminator: Option<u8>,
    top_level_stop: TopLevelStop<'_>,
    nesting_pairs: &[(u8, u8)],
) -> Option<usize> {
    let bytes = source.as_bytes();

    while index < bytes.len() {
        let byte = bytes[index];

        if let Some(close) = terminator {
            if byte == close {
                return Some(index);
            }
        } else if should_stop_top_level(top_level_stop, source, index, byte) {
            return Some(index);
        }

        index = match byte {
            b'\'' | b'"' => skip_string(bytes, index)?,
            b'`' => skip_template_literal(source, index)?,
            b'/' if index + 1 < bytes.len() && bytes[index + 1] == b'/' => {
                skip_line_comment(bytes, index)
            }
            b'/' if index + 1 < bytes.len() && bytes[index + 1] == b'*' => {
                skip_block_comment(bytes, index)?
            }
            _ => {
                if let Some(close) = close_for_open(nesting_pairs, byte) {
                    scan(
                        source,
                        index + 1,
                        Some(close),
                        top_level_stop,
                        nesting_pairs,
                    )?
                } else {
                    index
                }
            }
        };

        index += 1;
    }

    None
}
