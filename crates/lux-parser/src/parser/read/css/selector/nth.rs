use super::super::parser::CssParser;

pub fn is_nth_start(parser: &CssParser<'_>) -> bool {
    let remaining = parser.remaining();
    if remaining.starts_with("even") || remaining.starts_with("odd") {
        return true;
    }

    let bytes = remaining.as_bytes();
    if bytes.is_empty() {
        return false;
    }

    let b = bytes[0];
    b == b'+' || b == b'-' || b.is_ascii_digit()
}

pub fn read_nth<'a>(parser: &mut CssParser<'a>) -> &'a str {
    let start = parser.index;
    let bytes = parser.source.as_bytes();

    while parser.index < bytes.len() {
        let b = bytes[parser.index];
        if b.is_ascii_alphanumeric()
            || b == b'+'
            || b == b'-'
            || b == b'n'
            || b.is_ascii_whitespace()
        {
            parser.index += 1;
        } else {
            break;
        }
    }

    parser.source[start..parser.index].trim_end()
}

pub fn is_percentage_start(parser: &CssParser<'_>) -> bool {
    let bytes = parser.remaining().as_bytes();
    !bytes.is_empty() && bytes[0].is_ascii_digit()
}

pub fn read_percentage<'a>(parser: &mut CssParser<'a>) -> &'a str {
    let start = parser.index;
    let bytes = parser.source.as_bytes();

    while parser.index < bytes.len()
        && (bytes[parser.index].is_ascii_digit() || bytes[parser.index] == b'.')
    {
        parser.index += 1;
    }
    if parser.index < bytes.len() && bytes[parser.index] == b'%' {
        parser.index += 1;
    }

    &parser.source[start..parser.index]
}
