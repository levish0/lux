pub(super) fn sanitize_comment(data: &str) -> String {
    let mut sanitized = data.replace("--", "- -");
    if sanitized.ends_with('-') {
        sanitized.push(' ');
    }
    sanitized
}

pub(super) fn push_dynamic_marker(kind: &str, out: &mut String, has_dynamic: &mut bool) {
    *has_dynamic = true;
    out.push_str("<!--lux:dynamic:");
    out.push_str(kind);
    out.push_str("-->");
}
