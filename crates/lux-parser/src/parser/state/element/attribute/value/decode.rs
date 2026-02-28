use std::borrow::Cow;

use lux_ast::common::Span;
use lux_ast::template::tag::Text;
use lux_utils::html_entities::decode_character_references;

pub(super) fn decode_attr_text<'a>(
    raw: &'a str,
    span: Span,
    allocator: &'a oxc_allocator::Allocator,
) -> Text<'a> {
    let decoded = decode_character_references(raw, true);
    let data = match decoded {
        Cow::Borrowed(_) => raw,
        Cow::Owned(s) => allocator.alloc_str(&s),
    };

    Text { span, data, raw }
}
