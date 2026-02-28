use super::super::parser::CssParser;

pub fn read_matcher<'a>(parser: &mut CssParser<'a>) -> Option<&'a str> {
    let start = parser.index;
    let remaining = parser.remaining();

    if remaining.starts_with("~=")
        || remaining.starts_with("^=")
        || remaining.starts_with("$=")
        || remaining.starts_with("*=")
        || remaining.starts_with("|=")
    {
        parser.index += 2;
        Some(&parser.source[start..parser.index])
    } else if remaining.starts_with('=') {
        parser.index += 1;
        Some(&parser.source[start..parser.index])
    } else {
        None
    }
}

pub fn read_attr_flags<'a>(parser: &mut CssParser<'a>) -> Option<&'a str> {
    let start = parser.index;
    while parser.index < parser.source.len()
        && parser.source.as_bytes()[parser.index].is_ascii_alphabetic()
    {
        parser.index += 1;
    }

    if parser.index > start {
        Some(&parser.source[start..parser.index])
    } else {
        None
    }
}
