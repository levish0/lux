pub(super) fn sanitize_comment(data: &str) -> String {
    let mut sanitized = data.replace("--", "- -");
    if sanitized.ends_with('-') {
        sanitized.push(' ');
    }
    sanitized
}
