use std::sync::LazyLock;

use regex::Regex;

static SCRIPT_LANG_OR_COMMENT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)<!--.*?-->|<script\s+(?:[^>]*|(?:[^=>'"/]+=(?:"[^"]*"|'[^']*'|[^>\s]+)\s+)*)lang\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"' >]+))[^>]*>"#,
    )
    .expect("valid script lang/comment regex")
});

/// Detect whether a Svelte component should parse JS as TypeScript.
///
/// This matches Svelte's parse-time behavior conceptually:
/// scan source for `<script ... lang="ts">` (ignoring HTML comments).
pub fn detect_typescript_lang(template: &str) -> bool {
    for captures in SCRIPT_LANG_OR_COMMENT_RE.captures_iter(template) {
        let Some(token_match) = captures.get(0) else {
            continue;
        };
        let token = token_match.as_str();

        // Comment token starts with "<!", script token starts with "<s"/"<S".
        if token
            .as_bytes()
            .get(1)
            .is_none_or(|b| b.to_ascii_lowercase() != b's')
        {
            continue;
        }

        let value = captures
            .get(1)
            .or_else(|| captures.get(2))
            .or_else(|| captures.get(3))
            .map(|m| m.as_str().trim());

        if value.is_some_and(|v| v.eq_ignore_ascii_case("ts")) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::detect_typescript_lang;

    #[test]
    fn detects_lang_ts_quoted() {
        assert!(detect_typescript_lang(
            "<script lang=\"ts\">let x: number = 1;</script>"
        ));
    }

    #[test]
    fn detects_lang_ts_unquoted() {
        assert!(detect_typescript_lang(
            "<script lang=ts>let x = 1;</script>"
        ));
    }

    #[test]
    fn ignores_non_ts_lang() {
        assert!(!detect_typescript_lang(
            "<script lang=\"js\">let x = 1;</script>"
        ));
    }

    #[test]
    fn ignores_commented_out_script() {
        assert!(!detect_typescript_lang(
            "<!-- <script lang=\"ts\"></script> --><script>let x=1;</script>"
        ));
    }

    #[test]
    fn handles_script_with_other_attrs() {
        assert!(detect_typescript_lang(
            "<script context=\"module\" data-x='1' lang='ts'>export const x = 1;</script>"
        ));
    }

    #[test]
    fn handles_uppercase_lang_value() {
        assert!(detect_typescript_lang(
            "<script lang=\"TS\">let x: number = 1;</script>"
        ));
    }
}
