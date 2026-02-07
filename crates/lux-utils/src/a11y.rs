/// Accessibility constants (ARIA roles, attributes).
///
/// Reference: `2-analyze/visitors/shared/a11y/constants.js`
use phf::{phf_map, phf_set};

/// All valid ARIA attributes (without the `aria-` prefix).
pub static ARIA_ATTRIBUTES: phf::Set<&str> = phf_set! {
    "activedescendant", "atomic", "autocomplete", "busy", "checked",
    "colcount", "colindex", "colspan", "controls", "current",
    "describedby", "description", "details", "disabled", "dropeffect",
    "errormessage", "expanded", "flowto", "grabbed", "haspopup",
    "hidden", "invalid", "keyshortcuts", "label", "labelledby",
    "level", "live", "modal", "multiline", "multiselectable",
    "orientation", "owns", "placeholder", "posinset", "pressed",
    "readonly", "relevant", "required", "roledescription",
    "rowcount", "rowindex", "rowspan", "selected", "setsize",
    "sort", "valuemax", "valuemin", "valuenow", "valuetext",
};

pub fn is_valid_aria_attribute(name: &str) -> bool {
    ARIA_ATTRIBUTES.contains(name)
}

/// Elements that are invisible to the accessibility tree.
pub static INVISIBLE_ELEMENTS: phf::Set<&str> = phf_set! {
    "meta", "html", "script", "style",
};

/// Elements that are considered distracting and should be avoided.
pub static DISTRACTING_ELEMENTS: phf::Set<&str> = phf_set! {
    "blink", "marquee",
};

/// Roles that remove an element from the accessibility tree.
pub static PRESENTATION_ROLES: phf::Set<&str> = phf_set! {
    "presentation", "none",
};

/// Map of element name to its implicit ARIA role.
pub static IMPLICIT_ROLES: phf::Map<&str, &str> = phf_map! {
    "article" => "article",
    "aside" => "complementary",
    "body" => "document",
    "button" => "button",
    "datalist" => "listbox",
    "dd" => "definition",
    "dfn" => "term",
    "dialog" => "dialog",
    "details" => "group",
    "dt" => "term",
    "fieldset" => "group",
    "figure" => "figure",
    "form" => "form",
    "h1" => "heading",
    "h2" => "heading",
    "h3" => "heading",
    "h4" => "heading",
    "h5" => "heading",
    "h6" => "heading",
    "hr" => "separator",
    "img" => "img",
    "li" => "listitem",
    "link" => "link",
    "main" => "main",
    "menu" => "list",
    "meter" => "progressbar",
    "nav" => "navigation",
    "ol" => "list",
    "option" => "option",
    "optgroup" => "group",
    "output" => "status",
    "progress" => "progressbar",
    "section" => "region",
    "summary" => "button",
    "table" => "table",
    "tbody" => "rowgroup",
    "textarea" => "textbox",
    "tfoot" => "rowgroup",
    "thead" => "rowgroup",
    "tr" => "row",
    "ul" => "list",
};

/// Get the implicit ARIA role for an element.
pub fn get_implicit_role(element: &str) -> Option<&'static str> {
    IMPLICIT_ROLES.get(element).copied()
}

/// Map of input type to ARIA role.
pub static INPUT_TYPE_ROLES: phf::Map<&str, &str> = phf_map! {
    "button" => "button",
    "image" => "button",
    "reset" => "button",
    "submit" => "button",
    "checkbox" => "checkbox",
    "radio" => "radio",
    "range" => "slider",
    "number" => "spinbutton",
    "email" => "textbox",
    "search" => "textbox",
    "tel" => "textbox",
    "text" => "textbox",
    "url" => "textbox",
};

/// Get the ARIA role for an `<input>` element based on its type.
pub fn get_input_role(input_type: &str) -> Option<&'static str> {
    INPUT_TYPE_ROLES.get(input_type).copied()
}

/// All valid ARIA roles.
pub static ARIA_ROLES: phf::Set<&str> = phf_set! {
    "alert", "alertdialog", "application", "article", "banner",
    "blockquote", "button", "caption", "cell", "checkbox",
    "code", "columnheader", "combobox", "command", "comment",
    "complementary", "composite", "contentinfo", "definition",
    "deletion", "dialog", "directory", "document", "emphasis",
    "feed", "figure", "form", "generic", "grid",
    "gridcell", "group", "heading", "img", "input",
    "insertion", "landmark", "link", "list", "listbox",
    "listitem", "log", "main", "marquee", "math",
    "menu", "menubar", "menuitem", "menuitemcheckbox", "menuitemradio",
    "meter", "navigation", "none", "note", "option",
    "paragraph", "presentation", "progressbar", "radio", "radiogroup",
    "range", "region", "roletype", "row", "rowgroup",
    "rowheader", "scrollbar", "search", "searchbox", "section",
    "sectionhead", "select", "separator", "slider", "spinbutton",
    "status", "strong", "structure", "subscript", "superscript",
    "switch", "tab", "table", "tablist", "tabpanel",
    "term", "textbox", "time", "timer", "toolbar",
    "tooltip", "tree", "treegrid", "treeitem", "widget",
    "window",
};

pub fn is_valid_aria_role(role: &str) -> bool {
    ARIA_ROLES.contains(role)
}

/// Abstract ARIA roles that should not be used directly.
pub static ABSTRACT_ROLES: phf::Set<&str> = phf_set! {
    "command", "composite", "input", "landmark", "range",
    "roletype", "section", "sectionhead", "select", "structure",
    "widget", "window",
};

pub fn is_abstract_role(role: &str) -> bool {
    ABSTRACT_ROLES.contains(role)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aria_attributes() {
        assert!(is_valid_aria_attribute("label"));
        assert!(is_valid_aria_attribute("hidden"));
        assert!(is_valid_aria_attribute("describedby"));
        assert!(!is_valid_aria_attribute("nonexistent"));
    }

    #[test]
    fn test_implicit_roles() {
        assert_eq!(get_implicit_role("button"), Some("button"));
        assert_eq!(get_implicit_role("nav"), Some("navigation"));
        assert_eq!(get_implicit_role("h1"), Some("heading"));
        assert_eq!(get_implicit_role("div"), None);
    }

    #[test]
    fn test_input_roles() {
        assert_eq!(get_input_role("checkbox"), Some("checkbox"));
        assert_eq!(get_input_role("text"), Some("textbox"));
        assert_eq!(get_input_role("range"), Some("slider"));
        assert_eq!(get_input_role("hidden"), None);
    }

    #[test]
    fn test_aria_roles() {
        assert!(is_valid_aria_role("button"));
        assert!(is_valid_aria_role("dialog"));
        assert!(is_valid_aria_role("navigation"));
        assert!(!is_valid_aria_role("nonexistent"));
    }

    #[test]
    fn test_abstract_roles() {
        assert!(is_abstract_role("command"));
        assert!(is_abstract_role("widget"));
        assert!(!is_abstract_role("button"));
    }
}
