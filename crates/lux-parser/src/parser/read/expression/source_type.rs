use oxc_span::SourceType;

pub(super) fn make_source_type(ts: bool) -> SourceType {
    if ts {
        SourceType::ts()
    } else {
        SourceType::mjs()
    }
}
