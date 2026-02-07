/// HTML implicit closing tag rules.
///
/// Reference: `html-tree-validation.js`

/// Check if `current` element's closing tag is omitted when `next` sibling appears.
///
/// For example, `<li>a<li>b` — the first `<li>` is auto-closed when the second opens.
/// If `next` is `None`, we're at the end of the parent element.
pub fn closing_tag_omitted(current: &str, next: Option<&str>) -> bool {
    match current {
        "li" => matches!(next, Some("li") | None),
        "dt" => matches!(next, Some("dt" | "dd") | None),
        "dd" => matches!(next, Some("dt" | "dd") | None),
        "p" => matches!(
            next,
            Some(
                "address" | "article" | "aside" | "blockquote" | "div" | "dl"
                    | "fieldset" | "footer" | "form" | "h1" | "h2" | "h3" | "h4"
                    | "h5" | "h6" | "header" | "hgroup" | "hr" | "main" | "menu"
                    | "nav" | "ol" | "p" | "pre" | "section" | "table" | "ul"
            ) | None
        ),
        "rt" | "rp" => matches!(next, Some("rt" | "rp") | None),
        "optgroup" => matches!(next, Some("optgroup") | None),
        "option" => matches!(next, Some("option" | "optgroup") | None),
        "thead" | "tbody" => matches!(next, Some("tbody" | "tfoot") | None),
        "tfoot" => matches!(next, Some("tbody") | None),
        "tr" => matches!(next, Some("tr" | "tbody") | None),
        "td" | "th" => matches!(next, Some("td" | "th" | "tr") | None),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_li() {
        assert!(closing_tag_omitted("li", Some("li")));
        assert!(closing_tag_omitted("li", None));
        assert!(!closing_tag_omitted("li", Some("div")));
    }

    #[test]
    fn test_p() {
        assert!(closing_tag_omitted("p", Some("div")));
        assert!(closing_tag_omitted("p", Some("h1")));
        assert!(closing_tag_omitted("p", Some("p")));
        assert!(closing_tag_omitted("p", None));
        assert!(!closing_tag_omitted("p", Some("span")));
        assert!(!closing_tag_omitted("p", Some("a")));
    }

    #[test]
    fn test_table_elements() {
        assert!(closing_tag_omitted("td", Some("td")));
        assert!(closing_tag_omitted("td", Some("th")));
        assert!(closing_tag_omitted("td", Some("tr")));
        assert!(closing_tag_omitted("td", None));
        assert!(!closing_tag_omitted("td", Some("div")));
        assert!(closing_tag_omitted("tr", Some("tr")));
        assert!(closing_tag_omitted("thead", Some("tbody")));
    }

    #[test]
    fn test_non_omittable() {
        assert!(!closing_tag_omitted("div", Some("div")));
        assert!(!closing_tag_omitted("span", None));
    }
}
