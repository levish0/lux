//! HTML nesting validation.
//!
//! Reference: `html-tree-validation.js`

/// Validate that `child` tag is allowed as a direct child of `parent` tag.
///
/// Returns `None` if valid, `Some(message)` if invalid.
pub fn is_tag_valid_with_parent(child: &str, parent: &str) -> Option<&'static str> {
    match child {
        // Direct parent restrictions
        "tr" => {
            if !matches!(parent, "thead" | "tbody" | "tfoot" | "table" | "template") {
                return Some("<tr> must be child of <thead>, <tbody>, <tfoot>, or <table>");
            }
        }
        "td" | "th" => {
            if !matches!(parent, "tr" | "template") {
                return Some("<td>/<th> must be child of <tr>");
            }
        }
        "li" => {
            if !matches!(parent, "ul" | "ol" | "menu" | "template") {
                return Some("<li> must be child of <ul>, <ol>, or <menu>");
            }
        }
        "dd" | "dt" => {
            if !matches!(parent, "dl" | "template") {
                return Some("<dd>/<dt> must be child of <dl>");
            }
        }
        "caption" | "colgroup" | "thead" | "tbody" | "tfoot" => {
            if !matches!(parent, "table" | "template") {
                return Some("must be child of <table>");
            }
        }
        "col" => {
            if !matches!(parent, "colgroup" | "template") {
                return Some("<col> must be child of <colgroup>");
            }
        }
        "head" => {
            if parent != "html" {
                return Some("<head> must be child of <html>");
            }
        }
        "body" => {
            if parent != "html" {
                return Some("<body> must be child of <html>");
            }
        }
        _ => {}
    }

    // Parent restrictions — elements that cannot contain certain children
    match parent {
        // <p> can only contain phrasing content
        "p" => match child {
            "address" | "article" | "aside" | "blockquote" | "center" | "details" | "dialog"
            | "dir" | "div" | "dl" | "fieldset" | "figcaption" | "figure" | "footer" | "form"
            | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "header" | "hgroup" | "hr" | "listing"
            | "main" | "menu" | "nav" | "ol" | "p" | "plaintext" | "pre" | "search" | "section"
            | "summary" | "table" | "ul" | "xmp" => {
                return Some("this element cannot be child of <p>");
            }
            _ => {}
        },
        // Heading elements cannot contain other headings
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            if matches!(child, "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
                return Some("heading elements cannot contain other headings");
            }
        }
        // <select> can only contain <option>, <optgroup>, <hr>
        "select" => {
            if !matches!(child, "option" | "optgroup" | "hr" | "script" | "template") {
                return Some("this element cannot be child of <select>");
            }
        }
        // <optgroup> can only contain <option>
        "optgroup" => {
            if !matches!(child, "option" | "script" | "template") {
                return Some("this element cannot be child of <optgroup>");
            }
        }
        // <table> can only contain specific children
        "table" => {
            if !matches!(
                child,
                "caption"
                    | "colgroup"
                    | "thead"
                    | "tbody"
                    | "tfoot"
                    | "tr"
                    | "script"
                    | "template"
                    | "style"
            ) {
                return Some("this element cannot be child of <table>");
            }
        }
        _ => {}
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_nesting() {
        assert!(is_tag_valid_with_parent("li", "ul").is_none());
        assert!(is_tag_valid_with_parent("li", "ol").is_none());
        assert!(is_tag_valid_with_parent("tr", "tbody").is_none());
        assert!(is_tag_valid_with_parent("td", "tr").is_none());
        assert!(is_tag_valid_with_parent("div", "div").is_none());
    }

    #[test]
    fn test_invalid_nesting() {
        assert!(is_tag_valid_with_parent("li", "div").is_some());
        assert!(is_tag_valid_with_parent("tr", "div").is_some());
        assert!(is_tag_valid_with_parent("td", "div").is_some());
        assert!(is_tag_valid_with_parent("col", "div").is_some());
    }

    #[test]
    fn test_p_restrictions() {
        assert!(is_tag_valid_with_parent("div", "p").is_some());
        assert!(is_tag_valid_with_parent("h1", "p").is_some());
        assert!(is_tag_valid_with_parent("table", "p").is_some());
        assert!(is_tag_valid_with_parent("span", "p").is_none());
        assert!(is_tag_valid_with_parent("a", "p").is_none());
    }

    #[test]
    fn test_heading_restrictions() {
        assert!(is_tag_valid_with_parent("h2", "h1").is_some());
        assert!(is_tag_valid_with_parent("h1", "h1").is_some());
    }

    #[test]
    fn test_table_restrictions() {
        assert!(is_tag_valid_with_parent("div", "table").is_some());
        assert!(is_tag_valid_with_parent("thead", "table").is_none());
        assert!(is_tag_valid_with_parent("tr", "table").is_none());
    }

    #[test]
    fn test_template_escape() {
        // <template> allows any child
        assert!(is_tag_valid_with_parent("tr", "template").is_none());
        assert!(is_tag_valid_with_parent("td", "template").is_none());
        assert!(is_tag_valid_with_parent("li", "template").is_none());
    }
}
