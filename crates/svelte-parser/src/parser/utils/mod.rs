//! Utility functions.
//! Port of reference `src/utils.js`.

use std::sync::LazyLock;
use regex::Regex;

static REGEX_RETURN_CHARACTERS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\r").unwrap());

/// Port of reference's `hash()`.
/// Uses UTF-16 code units (matching JS `charCodeAt`) and base36 output (matching JS `toString(36)`).
pub fn hash(input: &str) -> String {
    let s = REGEX_RETURN_CHARACTERS.replace_all(input, "");
    // Collect UTF-16 code units (matching JS's charCodeAt behavior)
    let code_units: Vec<u16> = s.encode_utf16().collect();
    let mut hash: u32 = 5381;
    let mut i = code_units.len();
    while i > 0 {
        i -= 1;
        hash = ((hash << 5).wrapping_sub(hash)) ^ (code_units[i] as u32);
    }
    to_base36(hash)
}

/// Convert u32 to base36 string (matching JS's `Number.prototype.toString(36)`).
fn to_base36(mut n: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    const CHARS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut result = Vec::new();
    while n > 0 {
        result.push(CHARS[(n % 36) as usize]);
        n /= 36;
    }
    result.reverse();
    String::from_utf8(result).unwrap()
}

const VOID_ELEMENT_NAMES: &[&str] = &[
    "area", "base", "br", "col", "command", "embed", "hr", "img",
    "input", "keygen", "link", "meta", "param", "source", "track", "wbr",
];

/// Returns `true` if `name` is of a void element.
pub fn is_void(name: &str) -> bool {
    VOID_ELEMENT_NAMES.contains(&name) || name.eq_ignore_ascii_case("!doctype")
}

const RESERVED_WORDS: &[&str] = &[
    "arguments", "await", "break", "case", "catch", "class", "const", "continue",
    "debugger", "default", "delete", "do", "else", "enum", "eval", "export",
    "extends", "false", "finally", "for", "function", "if", "implements", "import",
    "in", "instanceof", "interface", "let", "new", "null", "package", "private",
    "protected", "public", "return", "static", "super", "switch", "this", "throw",
    "true", "try", "typeof", "var", "void", "while", "with", "yield",
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
    "beforeinput", "click", "change", "dblclick", "contextmenu", "focusin", "focusout",
    "input", "keydown", "keyup", "mousedown", "mousemove", "mouseout", "mouseover",
    "mouseup", "pointerdown", "pointermove", "pointerout", "pointerover", "pointerup",
    "touchend", "touchmove", "touchstart",
];

/// Returns `true` if `event_name` is a delegated event.
pub fn can_delegate_event(event_name: &str) -> bool {
    DELEGATED_EVENTS.contains(&event_name)
}

const DOM_BOOLEAN_ATTRIBUTES: &[&str] = &[
    "allowfullscreen", "async", "autofocus", "autoplay", "checked", "controls",
    "default", "disabled", "formnovalidate", "indeterminate", "inert", "ismap",
    "loop", "multiple", "muted", "nomodule", "novalidate", "open", "playsinline",
    "readonly", "required", "reversed", "seamless", "selected", "webkitdirectory",
    "defer", "disablepictureinpicture", "disableremoteplayback",
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
    "allowfullscreen", "async", "autofocus", "autoplay", "checked", "controls",
    "default", "disabled", "formnovalidate", "indeterminate", "inert", "ismap",
    "loop", "multiple", "muted", "nomodule", "novalidate", "open", "playsinline",
    "readonly", "required", "reversed", "seamless", "selected", "webkitdirectory",
    "defer", "disablepictureinpicture", "disableremoteplayback",
    "formNoValidate", "isMap", "noModule", "playsInline", "readOnly",
    "value", "volume", "defaultValue", "defaultChecked", "srcObject",
    "noValidate", "allowFullscreen", "disablePictureInPicture", "disableRemotePlayback",
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
    "altGlyph", "altGlyphDef", "altGlyphItem", "animate", "animateColor",
    "animateMotion", "animateTransform", "circle", "clipPath", "color-profile",
    "cursor", "defs", "desc", "discard", "ellipse", "feBlend", "feColorMatrix",
    "feComponentTransfer", "feComposite", "feConvolveMatrix", "feDiffuseLighting",
    "feDisplacementMap", "feDistantLight", "feDropShadow", "feFlood", "feFuncA",
    "feFuncB", "feFuncG", "feFuncR", "feGaussianBlur", "feImage", "feMerge",
    "feMergeNode", "feMorphology", "feOffset", "fePointLight", "feSpecularLighting",
    "feSpotLight", "feTile", "feTurbulence", "filter", "font", "font-face",
    "font-face-format", "font-face-name", "font-face-src", "font-face-uri",
    "foreignObject", "g", "glyph", "glyphRef", "hatch", "hatchpath", "hkern",
    "image", "line", "linearGradient", "marker", "mask", "mesh", "meshgradient",
    "meshpatch", "meshrow", "metadata", "missing-glyph", "mpath", "path", "pattern",
    "polygon", "polyline", "radialGradient", "rect", "set", "solidcolor", "stop",
    "svg", "switch", "symbol", "text", "textPath", "tref", "tspan", "unknown",
    "use", "view", "vkern",
];

/// Returns `true` if `name` is an SVG element.
pub fn is_svg(name: &str) -> bool {
    SVG_ELEMENTS.contains(&name)
}

const MATHML_ELEMENTS: &[&str] = &[
    "annotation", "annotation-xml", "maction", "math", "merror", "mfrac", "mi",
    "mmultiscripts", "mn", "mo", "mover", "mpadded", "mphantom", "mprescripts",
    "mroot", "mrow", "ms", "mspace", "msqrt", "mstyle", "msub", "msubsup", "msup",
    "mtable", "mtd", "mtext", "mtr", "munder", "munderover", "semantics",
];

/// Returns `true` if `name` is a MathML element.
pub fn is_mathml(name: &str) -> bool {
    MATHML_ELEMENTS.contains(&name)
}

const STATE_CREATION_RUNES: &[&str] = &[
    "$state", "$state.raw", "$derived", "$derived.by",
];

const RUNES: &[&str] = &[
    "$state", "$state.raw", "$derived", "$derived.by",
    "$state.eager", "$state.snapshot", "$props", "$props.id",
    "$bindable", "$effect", "$effect.pre", "$effect.tracking",
    "$effect.root", "$effect.pending", "$inspect", "$inspect().with",
    "$inspect.trace", "$host",
];

/// Returns `true` if `name` is a rune.
pub fn is_rune(name: &str) -> bool {
    RUNES.contains(&name)
}

/// Returns `true` if `name` is a state creation rune.
pub fn is_state_creation_rune(name: &str) -> bool {
    STATE_CREATION_RUNES.contains(&name)
}

const RAW_TEXT_ELEMENTS: &[&str] = &["textarea", "script", "style", "title"];

/// Returns `true` if `name` is a raw text element.
pub fn is_raw_text_element(name: &str) -> bool {
    RAW_TEXT_ELEMENTS.contains(&name)
}
