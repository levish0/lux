//! Utility functions.
const VOID_ELEMENT_NAMES: &[&str] = &[
    "area", "base", "br", "col", "command", "embed", "hr", "img", "input", "keygen", "link",
    "meta", "param", "source", "track", "wbr",
];

/// Returns `true` if `name` is of a void element.
pub fn is_void(name: &str) -> bool {
    VOID_ELEMENT_NAMES.contains(&name) || name.eq_ignore_ascii_case("!doctype")
}

const RESERVED_WORDS: &[&str] = &[
    "arguments",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "eval",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "let",
    "new",
    "null",
    "package",
    "private",
    "protected",
    "public",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

/// Returns `true` if `word` is a reserved JavaScript keyword.
pub fn is_reserved(word: &str) -> bool {
    RESERVED_WORDS.contains(&word)
}

/// Returns `true` if the event name is a capture event.
pub fn is_capture_event(name: &str) -> bool {
    name.ends_with("capture") && name != "gotpointercapture" && name != "lostpointercapture"
}

const DELEGATED_EVENTS: &[&str] = &[
    "beforeinput",
    "click",
    "change",
    "dblclick",
    "contextmenu",
    "focusin",
    "focusout",
    "input",
    "keydown",
    "keyup",
    "mousedown",
    "mousemove",
    "mouseout",
    "mouseover",
    "mouseup",
    "pointerdown",
    "pointermove",
    "pointerout",
    "pointerover",
    "pointerup",
    "touchend",
    "touchmove",
    "touchstart",
];

/// Returns `true` if `event_name` is a delegated event.
pub fn can_delegate_event(event_name: &str) -> bool {
    DELEGATED_EVENTS.contains(&event_name)
}

const DOM_BOOLEAN_ATTRIBUTES: &[&str] = &[
    "allowfullscreen",
    "async",
    "autofocus",
    "autoplay",
    "checked",
    "controls",
    "default",
    "disabled",
    "formnovalidate",
    "indeterminate",
    "inert",
    "ismap",
    "loop",
    "multiple",
    "muted",
    "nomodule",
    "novalidate",
    "open",
    "playsinline",
    "readonly",
    "required",
    "reversed",
    "seamless",
    "selected",
    "webkitdirectory",
    "defer",
    "disablepictureinpicture",
    "disableremoteplayback",
];

/// Returns `true` if `name` is a boolean attribute.
pub fn is_boolean_attribute(name: &str) -> bool {
    DOM_BOOLEAN_ATTRIBUTES.contains(&name)
}

/// Attribute name aliases (attribute name → property name).
pub fn normalize_attribute(name: &str) -> &str {
    let lower = name;
    match lower {
        "formnovalidate" => "formNoValidate",
        "ismap" => "isMap",
        "nomodule" => "noModule",
        "playsinline" => "playsInline",
        "readonly" => "readOnly",
        "defaultvalue" => "defaultValue",
        "defaultchecked" => "defaultChecked",
        "srcobject" => "srcObject",
        "novalidate" => "noValidate",
        "allowfullscreen" => "allowFullscreen",
        "disablepictureinpicture" => "disablePictureInPicture",
        "disableremoteplayback" => "disableRemotePlayback",
        _ => lower,
    }
}

const DOM_PROPERTIES: &[&str] = &[
    "allowfullscreen",
    "async",
    "autofocus",
    "autoplay",
    "checked",
    "controls",
    "default",
    "disabled",
    "formnovalidate",
    "indeterminate",
    "inert",
    "ismap",
    "loop",
    "multiple",
    "muted",
    "nomodule",
    "novalidate",
    "open",
    "playsinline",
    "readonly",
    "required",
    "reversed",
    "seamless",
    "selected",
    "webkitdirectory",
    "defer",
    "disablepictureinpicture",
    "disableremoteplayback",
    "formNoValidate",
    "isMap",
    "noModule",
    "playsInline",
    "readOnly",
    "value",
    "volume",
    "defaultValue",
    "defaultChecked",
    "srcObject",
    "noValidate",
    "allowFullscreen",
    "disablePictureInPicture",
    "disableRemotePlayback",
];

/// Returns `true` if `name` is a DOM property.
pub fn is_dom_property(name: &str) -> bool {
    DOM_PROPERTIES.contains(&name)
}

const NON_STATIC_PROPERTIES: &[&str] = &["autofocus", "muted", "defaultValue", "defaultChecked"];

/// Returns `true` if the given attribute cannot be set through the template string.
pub fn cannot_be_set_statically(name: &str) -> bool {
    NON_STATIC_PROPERTIES.contains(&name)
}

const PASSIVE_EVENTS: &[&str] = &["touchstart", "touchmove"];

/// Returns `true` if `name` is a passive event.
pub fn is_passive_event(name: &str) -> bool {
    PASSIVE_EVENTS.contains(&name)
}

const CONTENT_EDITABLE_BINDINGS: &[&str] = &["textContent", "innerHTML", "innerText"];

/// Returns `true` if `name` is a content editable binding.
pub fn is_content_editable_binding(name: &str) -> bool {
    CONTENT_EDITABLE_BINDINGS.contains(&name)
}

const LOAD_ERROR_ELEMENTS: &[&str] = &[
    "body", "embed", "iframe", "img", "link", "object", "script", "style", "track",
];

/// Returns `true` if the element emits `load` and `error` events.
pub fn is_load_error_element(name: &str) -> bool {
    LOAD_ERROR_ELEMENTS.contains(&name)
}

const SVG_ELEMENTS: &[&str] = &[
    "altGlyph",
    "altGlyphDef",
    "altGlyphItem",
    "animate",
    "animateColor",
    "animateMotion",
    "animateTransform",
    "circle",
    "clipPath",
    "color-profile",
    "cursor",
    "defs",
    "desc",
    "discard",
    "ellipse",
    "feBlend",
    "feColorMatrix",
    "feComponentTransfer",
    "feComposite",
    "feConvolveMatrix",
    "feDiffuseLighting",
    "feDisplacementMap",
    "feDistantLight",
    "feDropShadow",
    "feFlood",
    "feFuncA",
    "feFuncB",
    "feFuncG",
    "feFuncR",
    "feGaussianBlur",
    "feImage",
    "feMerge",
    "feMergeNode",
    "feMorphology",
    "feOffset",
    "fePointLight",
    "feSpecularLighting",
    "feSpotLight",
    "feTile",
    "feTurbulence",
    "filter",
    "font",
    "font-face",
    "font-face-format",
    "font-face-name",
    "font-face-src",
    "font-face-uri",
    "foreignObject",
    "g",
    "glyph",
    "glyphRef",
    "hatch",
    "hatchpath",
    "hkern",
    "image",
    "line",
    "linearGradient",
    "marker",
    "mask",
    "mesh",
    "meshgradient",
    "meshpatch",
    "meshrow",
    "metadata",
    "missing-glyph",
    "mpath",
    "path",
    "pattern",
    "polygon",
    "polyline",
    "radialGradient",
    "rect",
    "set",
    "solidcolor",
    "stop",
    "svg",
    "switch",
    "symbol",
    "text",
    "textPath",
    "tref",
    "tspan",
    "unknown",
    "use",
    "view",
    "vkern",
];

/// Returns `true` if `name` is an SVG element.
pub fn is_svg(name: &str) -> bool {
    SVG_ELEMENTS.contains(&name)
}

const MATHML_ELEMENTS: &[&str] = &[
    "annotation",
    "annotation-xml",
    "maction",
    "math",
    "merror",
    "mfrac",
    "mi",
    "mmultiscripts",
    "mn",
    "mo",
    "mover",
    "mpadded",
    "mphantom",
    "mprescripts",
    "mroot",
    "mrow",
    "ms",
    "mspace",
    "msqrt",
    "mstyle",
    "msub",
    "msubsup",
    "msup",
    "mtable",
    "mtd",
    "mtext",
    "mtr",
    "munder",
    "munderover",
    "semantics",
];

/// Returns `true` if `name` is a MathML element.
pub fn is_mathml(name: &str) -> bool {
    MATHML_ELEMENTS.contains(&name)
}

const STATE_CREATION_RUNES: &[&str] = &["$state", "$state.raw", "$derived", "$derived.by"];

const RUNES: &[&str] = &[
    "$state",
    "$state.raw",
    "$derived",
    "$derived.by",
    "$state.eager",
    "$state.snapshot",
    "$props",
    "$props.id",
    "$bindable",
    "$effect",
    "$effect.pre",
    "$effect.tracking",
    "$effect.root",
    "$effect.pending",
    "$inspect",
    "$inspect().with",
    "$inspect.trace",
    "$host",
];

/// Returns `true` if `name` is a rune.
pub fn is_rune(name: &str) -> bool {
    RUNES.contains(&name)
}

/// Returns `true` if `name` is a state creation rune.
pub fn is_state_creation_rune(name: &str) -> bool {
    STATE_CREATION_RUNES.contains(&name)
}

// ─── HTML Tree Validation ────────────────────────────────────────
// Port of reference `src/html-tree-validation.js`

/// Autoclosing rules: `direct` means only immediate children trigger auto-close,
/// `descendant` means any descendant triggers auto-close.
enum AutoCloseRule {
    Direct(&'static [&'static str]),
    Descendant(&'static [&'static str]),
}

fn autoclosing_rule(tag: &str) -> Option<AutoCloseRule> {
    match tag {
        "li" => Some(AutoCloseRule::Direct(&["li"])),
        "dt" => Some(AutoCloseRule::Descendant(&["dt", "dd"])),
        "dd" => Some(AutoCloseRule::Descendant(&["dt", "dd"])),
        "p" => Some(AutoCloseRule::Descendant(&[
            "address", "article", "aside", "blockquote", "div", "dl",
            "fieldset", "footer", "form", "h1", "h2", "h3", "h4",
            "h5", "h6", "header", "hgroup", "hr", "main", "menu",
            "nav", "ol", "p", "pre", "section", "table", "ul",
        ])),
        "rt" => Some(AutoCloseRule::Descendant(&["rt", "rp"])),
        "rp" => Some(AutoCloseRule::Descendant(&["rt", "rp"])),
        "optgroup" => Some(AutoCloseRule::Descendant(&["optgroup"])),
        "option" => Some(AutoCloseRule::Descendant(&["option", "optgroup"])),
        "thead" => Some(AutoCloseRule::Direct(&["tbody", "tfoot"])),
        "tbody" => Some(AutoCloseRule::Direct(&["tbody", "tfoot"])),
        "tfoot" => Some(AutoCloseRule::Direct(&["tbody"])),
        "tr" => Some(AutoCloseRule::Direct(&["tr", "tbody"])),
        "td" => Some(AutoCloseRule::Direct(&["td", "th", "tr"])),
        "th" => Some(AutoCloseRule::Direct(&["td", "th", "tr"])),
        _ => None,
    }
}

/// Returns true if the `current` tag should be auto-closed when `next` tag is encountered.
/// Port of reference `closing_tag_omitted` from html-tree-validation.js.
pub fn closing_tag_omitted(current: &str, next: &str) -> bool {
    if let Some(rule) = autoclosing_rule(current) {
        let list = match rule {
            AutoCloseRule::Direct(tags) | AutoCloseRule::Descendant(tags) => tags,
        };
        return list.contains(&next);
    }
    false
}

/// Returns an error message if the tag is not allowed inside the parent tag.
/// Port of reference `is_tag_valid_with_parent` from html-tree-validation.js.
pub fn is_tag_valid_with_parent(child_tag: &str, parent_tag: &str) -> Option<String> {
    if child_tag.contains('-') || parent_tag.contains('-') {
        return None; // custom elements can be anything
    }
    if parent_tag == "template" {
        return None;
    }

    // Check disallowed_children rules
    if let Some(msg) = check_disallowed(child_tag, parent_tag) {
        return Some(msg);
    }

    // Tags only valid with specific parents
    match child_tag {
        "body" | "caption" | "col" | "colgroup" | "frameset" | "frame" | "head" | "html" => {
            Some(format!("`<{child_tag}>` cannot be a child of `<{parent_tag}>`"))
        }
        "thead" | "tbody" | "tfoot" => {
            Some(format!("`<{child_tag}>` must be the child of a `<table>`, not a `<{parent_tag}>`"))
        }
        "td" | "th" => {
            Some(format!("`<{child_tag}>` must be the child of a `<tr>`, not a `<{parent_tag}>`"))
        }
        "tr" => Some(format!(
            "`<tr>` must be the child of a `<thead>`, `<tbody>`, or `<tfoot>`, not a `<{parent_tag}>`"
        )),
        _ => None,
    }
}

/// Check disallowed_children rules (superset of autoclosing_children).
fn check_disallowed(child_tag: &str, parent_tag: &str) -> Option<String> {
    // 'only' rules — certain parents only allow specific children
    let only_children: Option<&[&str]> = match parent_tag {
        "tr" => Some(&["th", "td", "style", "script", "template"]),
        "tbody" | "thead" | "tfoot" => Some(&["tr", "style", "script", "template"]),
        "colgroup" => Some(&["col", "template"]),
        "table" => Some(&["caption", "colgroup", "tbody", "thead", "tfoot", "style", "script", "template"]),
        "head" => Some(&["base", "basefont", "bgsound", "link", "meta", "title", "noscript", "noframes", "style", "script", "template"]),
        "html" => Some(&["head", "body", "frameset"]),
        "frameset" => Some(&["frame"]),
        _ => None,
    };

    if let Some(allowed) = only_children {
        if allowed.contains(&child_tag) {
            return None; // allowed
        } else {
            return Some(format!(
                "`<{child_tag}>` cannot be a child of `<{parent_tag}>`"
            ));
        }
    }

    // 'direct' rules (from autoclosing + disallowed)
    let direct_disallowed: Option<&[&str]> = match parent_tag {
        "li" => Some(&["li"]),
        "thead" => Some(&["tbody", "tfoot"]),
        "tbody" => Some(&["tbody", "tfoot"]),
        "tfoot" => Some(&["tbody"]),
        "tr" => Some(&["tr", "tbody"]),
        "td" => Some(&["td", "th", "tr"]),
        "th" => Some(&["td", "th", "tr"]),
        _ => None,
    };

    if let Some(list) = direct_disallowed {
        if list.contains(&child_tag) {
            return Some(format!(
                "`<{child_tag}>` cannot be a direct child of `<{parent_tag}>`"
            ));
        }
    }

    // 'descendant' rules
    let descendant_disallowed: Option<&[&str]> = match parent_tag {
        "dt" | "dd" => Some(&["dt", "dd"]),
        "p" => Some(&[
            "address", "article", "aside", "blockquote", "div", "dl",
            "fieldset", "footer", "form", "h1", "h2", "h3", "h4",
            "h5", "h6", "header", "hgroup", "hr", "main", "menu",
            "nav", "ol", "p", "pre", "section", "table", "ul",
        ]),
        "rt" | "rp" => Some(&["rt", "rp"]),
        "optgroup" => Some(&["optgroup"]),
        "option" => Some(&["option", "optgroup"]),
        "form" => Some(&["form"]),
        "a" => Some(&["a"]),
        "button" => Some(&["button"]),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            Some(&["h1", "h2", "h3", "h4", "h5", "h6"])
        }
        _ => None,
    };

    if let Some(list) = descendant_disallowed {
        if list.contains(&child_tag) {
            return Some(format!(
                "`<{child_tag}>` cannot be a child of `<{parent_tag}>`"
            ));
        }
    }

    None
}

const RAW_TEXT_ELEMENTS: &[&str] = &["textarea", "script", "style", "title"];

/// Returns `true` if `name` is a raw text element.
pub fn is_raw_text_element(name: &str) -> bool {
    RAW_TEXT_ELEMENTS.contains(&name)
}
