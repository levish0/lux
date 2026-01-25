//! A11y constants for accessibility validation.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/a11y/constants.js`
//!
//! This module contains constant data used for accessibility validation.
//! Some of the reference implementation derives data dynamically from aria-query
//! and axobject-query packages. We define the static equivalents here.

use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::LazyLock;

/// Valid ARIA attributes (without the "aria-" prefix).
pub static ARIA_ATTRIBUTES: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    [
        "activedescendant", "atomic", "autocomplete", "busy", "checked",
        "colcount", "colindex", "colspan", "controls", "current",
        "describedby", "description", "details", "disabled", "dropeffect",
        "errormessage", "expanded", "flowto", "grabbed", "haspopup",
        "hidden", "invalid", "keyshortcuts", "label", "labelledby",
        "level", "live", "modal", "multiline", "multiselectable",
        "orientation", "owns", "placeholder", "posinset", "pressed",
        "readonly", "relevant", "required", "roledescription", "rowcount",
        "rowindex", "rowspan", "selected", "setsize", "sort",
        "valuemax", "valuemin", "valuenow", "valuetext",
    ].into_iter().collect()
});

/// Required attributes for specific elements.
pub static A11Y_REQUIRED_ATTRIBUTES: LazyLock<FxHashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    let mut map = FxHashMap::default();
    map.insert("a", vec!["href"]);
    map.insert("area", vec!["alt", "aria-label", "aria-labelledby"]);
    map.insert("html", vec!["lang"]);
    map.insert("iframe", vec!["title"]);
    map.insert("img", vec!["alt"]);
    map.insert("object", vec!["title", "aria-label", "aria-labelledby"]);
    map
});

/// Distracting elements that should be avoided.
pub static A11Y_DISTRACTING_ELEMENTS: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    ["blink", "marquee"].into_iter().collect()
});

/// Elements that require text content (headings).
pub static A11Y_REQUIRED_CONTENT: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    ["h1", "h2", "h3", "h4", "h5", "h6"].into_iter().collect()
});

/// Elements that can be associated with a label.
pub static A11Y_LABELABLE: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    [
        "button", "input", "keygen", "meter",
        "output", "progress", "select", "textarea",
    ].into_iter().collect()
});

/// Event handlers considered interactive.
pub static A11Y_INTERACTIVE_HANDLERS: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    [
        // Keyboard events
        "keypress", "keydown", "keyup",
        // Click events
        "click", "contextmenu", "dblclick",
        "drag", "dragend", "dragenter", "dragexit",
        "dragleave", "dragover", "dragstart", "drop",
        "mousedown", "mouseenter", "mouseleave",
        "mousemove", "mouseout", "mouseover", "mouseup",
    ].into_iter().collect()
});

/// Recommended interactive handlers for a11y checks.
pub static A11Y_RECOMMENDED_INTERACTIVE_HANDLERS: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    ["click", "mousedown", "mouseup", "keypress", "keydown", "keyup"].into_iter().collect()
});

/// Nested implicit semantics (element -> role when nested in section/article).
pub static A11Y_NESTED_IMPLICIT_SEMANTICS: LazyLock<FxHashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut map = FxHashMap::default();
    map.insert("header", "banner");
    map.insert("footer", "contentinfo");
    map
});

/// Implicit semantics mapping (element -> role).
pub static A11Y_IMPLICIT_SEMANTICS: LazyLock<FxHashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut map = FxHashMap::default();
    map.insert("a", "link");
    map.insert("area", "link");
    map.insert("article", "article");
    map.insert("aside", "complementary");
    map.insert("body", "document");
    map.insert("button", "button");
    map.insert("datalist", "listbox");
    map.insert("dd", "definition");
    map.insert("dfn", "term");
    map.insert("dialog", "dialog");
    map.insert("details", "group");
    map.insert("dt", "term");
    map.insert("fieldset", "group");
    map.insert("figure", "figure");
    map.insert("form", "form");
    map.insert("h1", "heading");
    map.insert("h2", "heading");
    map.insert("h3", "heading");
    map.insert("h4", "heading");
    map.insert("h5", "heading");
    map.insert("h6", "heading");
    map.insert("hr", "separator");
    map.insert("img", "img");
    map.insert("li", "listitem");
    map.insert("link", "link");
    map.insert("main", "main");
    map.insert("menu", "list");
    map.insert("meter", "progressbar");
    map.insert("nav", "navigation");
    map.insert("ol", "list");
    map.insert("option", "option");
    map.insert("optgroup", "group");
    map.insert("output", "status");
    map.insert("progress", "progressbar");
    map.insert("section", "region");
    map.insert("summary", "button");
    map.insert("table", "table");
    map.insert("tbody", "rowgroup");
    map.insert("textarea", "textbox");
    map.insert("tfoot", "rowgroup");
    map.insert("thead", "rowgroup");
    map.insert("tr", "row");
    map.insert("ul", "list");
    map
});

/// Menuitem type to implicit role mapping.
pub static MENUITEM_TYPE_TO_IMPLICIT_ROLE: LazyLock<FxHashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut map = FxHashMap::default();
    map.insert("command", "menuitem");
    map.insert("checkbox", "menuitemcheckbox");
    map.insert("radio", "menuitemradio");
    map
});

/// Input type to implicit role mapping.
pub static INPUT_TYPE_TO_IMPLICIT_ROLE: LazyLock<FxHashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut map = FxHashMap::default();
    map.insert("button", "button");
    map.insert("image", "button");
    map.insert("reset", "button");
    map.insert("submit", "button");
    map.insert("checkbox", "checkbox");
    map.insert("radio", "radio");
    map.insert("range", "slider");
    map.insert("number", "spinbutton");
    map.insert("email", "textbox");
    map.insert("search", "searchbox");
    map.insert("tel", "textbox");
    map.insert("text", "textbox");
    map.insert("url", "textbox");
    map
});

/// Exceptions for non-interactive elements that can have interactive roles.
pub static A11Y_NON_INTERACTIVE_ELEMENT_TO_INTERACTIVE_ROLE_EXCEPTIONS: LazyLock<FxHashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
    let mut map = FxHashMap::default();
    map.insert("ul", vec!["listbox", "menu", "menubar", "radiogroup", "tablist", "tree", "treegrid"]);
    map.insert("ol", vec!["listbox", "menu", "menubar", "radiogroup", "tablist", "tree", "treegrid"]);
    map.insert("li", vec!["menuitem", "option", "row", "tab", "treeitem"]);
    map.insert("table", vec!["grid"]);
    map.insert("td", vec!["gridcell"]);
    map.insert("fieldset", vec!["radiogroup", "presentation"]);
    map
});

/// Input types that become combobox when list attribute is present.
pub static COMBOBOX_IF_LIST: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    ["email", "search", "tel", "text", "url"].into_iter().collect()
});

/// Address type tokens for autocomplete.
pub static ADDRESS_TYPE_TOKENS: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    ["shipping", "billing"].into_iter().collect()
});

/// Autofill field name tokens for autocomplete.
pub static AUTOFILL_FIELD_NAME_TOKENS: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    [
        "", "on", "off", "name", "honorific-prefix", "given-name",
        "additional-name", "family-name", "honorific-suffix", "nickname",
        "username", "new-password", "current-password", "one-time-code",
        "organization-title", "organization", "street-address",
        "address-line1", "address-line2", "address-line3",
        "address-level4", "address-level3", "address-level2", "address-level1",
        "country", "country-name", "postal-code",
        "cc-name", "cc-given-name", "cc-additional-name", "cc-family-name",
        "cc-number", "cc-exp", "cc-exp-month", "cc-exp-year", "cc-csc", "cc-type",
        "transaction-currency", "transaction-amount", "language",
        "bday", "bday-day", "bday-month", "bday-year", "sex", "url", "photo",
    ].into_iter().collect()
});

/// Contact type tokens for autocomplete.
pub static CONTACT_TYPE_TOKENS: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    ["home", "work", "mobile", "fax", "pager"].into_iter().collect()
});

/// Autofill contact field name tokens for autocomplete.
pub static AUTOFILL_CONTACT_FIELD_NAME_TOKENS: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    [
        "tel", "tel-country-code", "tel-national", "tel-area-code",
        "tel-local", "tel-local-prefix", "tel-local-suffix", "tel-extension",
        "email", "impp",
    ].into_iter().collect()
});

/// Element interactivity classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementInteractivity {
    Interactive,
    NonInteractive,
    Static,
}

/// Invisible elements that don't support ARIA.
pub static INVISIBLE_ELEMENTS: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    ["meta", "html", "script", "style"].into_iter().collect()
});

/// Presentation roles (no semantic meaning).
pub static PRESENTATION_ROLES: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    ["presentation", "none"].into_iter().collect()
});

/// Abstract ARIA roles (cannot be used directly).
/// Reference: Derived from aria-query roles_map.
pub static ABSTRACT_ROLES: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    [
        "command", "composite", "input", "landmark", "range",
        "roletype", "section", "sectionhead", "select", "structure",
        "widget", "window",
    ].into_iter().collect()
});

/// All valid ARIA roles.
/// Reference: Derived from aria-query roles_map.
pub static ARIA_ROLES: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    [
        // Widget roles
        "button", "checkbox", "gridcell", "link", "menuitem",
        "menuitemcheckbox", "menuitemradio", "option", "progressbar",
        "radio", "scrollbar", "searchbox", "separator", "slider",
        "spinbutton", "switch", "tab", "tabpanel", "textbox", "treeitem",
        // Composite widget roles
        "combobox", "grid", "listbox", "menu", "menubar", "radiogroup",
        "tablist", "tree", "treegrid",
        // Document structure roles
        "application", "article", "blockquote", "caption", "cell",
        "columnheader", "definition", "deletion", "directory", "document",
        "emphasis", "feed", "figure", "generic", "group", "heading",
        "img", "insertion", "list", "listitem", "math", "meter", "none",
        "note", "paragraph", "presentation", "row", "rowgroup", "rowheader",
        "separator", "strong", "subscript", "superscript", "table", "term",
        "time", "toolbar", "tooltip",
        // Landmark roles
        "banner", "complementary", "contentinfo", "form", "main",
        "navigation", "region", "search",
        // Live region roles
        "alert", "log", "marquee", "status", "timer",
        // Window roles
        "alertdialog", "dialog",
        // Abstract roles (included for validation but should not be used)
        "command", "composite", "input", "landmark", "range",
        "roletype", "section", "sectionhead", "select", "structure",
        "widget", "window",
    ].into_iter().collect()
});

/// Non-interactive ARIA roles.
/// Reference: Derived from aria-query roles that don't descend from widget/window.
pub static NON_INTERACTIVE_ROLES: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    [
        "article", "banner", "blockquote", "caption", "cell",
        "columnheader", "complementary", "contentinfo", "definition",
        "deletion", "directory", "document", "emphasis", "feed",
        "figure", "form", "group", "heading", "img", "insertion",
        "list", "listitem", "log", "main", "marquee", "math",
        "meter", "navigation", "none", "note", "paragraph",
        "presentation", "region", "row", "rowgroup", "rowheader",
        "search", "status", "strong", "subscript", "superscript",
        "table", "term", "time", "timer", "tooltip",
        // progressbar is a special case - descendant of widget but readonly
        "progressbar",
    ].into_iter().collect()
});

/// Interactive ARIA roles.
/// Reference: Derived from aria-query roles that descend from widget/window.
pub static INTERACTIVE_ROLES: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    [
        "alert", "alertdialog", "application", "button", "checkbox",
        "combobox", "dialog", "grid", "gridcell", "link", "listbox",
        "menu", "menubar", "menuitem", "menuitemcheckbox", "menuitemradio",
        "option", "radio", "radiogroup", "scrollbar", "searchbox",
        "separator", "slider", "spinbutton", "switch", "tab", "tablist",
        "tabpanel", "textbox", "toolbar", "tree", "treegrid", "treeitem",
        // cell is treated as interactive by AXObject
        "cell",
    ].into_iter().collect()
});
