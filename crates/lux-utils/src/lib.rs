//! Utility functions for Svelte/Lux compiler.
//!
//! This module provides various helper functions ported from Svelte's utils.js.

use phf::phf_set;

/// Computes a hash of a string using DJB2 algorithm, returning base36.
/// Matches Svelte's hash function from utils.js.
pub fn hash(s: &str) -> String {
    // Remove carriage returns like the original
    let s = s.replace('\r', "");
    let mut hash: u32 = 5381;

    // Use bytes for closer matching to charCodeAt (ASCII range)
    for &b in s.as_bytes().iter().rev() {
        hash = ((hash << 5).wrapping_sub(hash)) ^ (b as u32);
    }

    // Convert to base36
    format_radix(hash, 36)
}

/// Formats a number in the given radix (base).
fn format_radix(mut n: u32, radix: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }

    let mut result = Vec::new();
    while n > 0 {
        let digit = (n % radix) as u8;
        let c = if digit < 10 {
            b'0' + digit
        } else {
            b'a' + (digit - 10)
        };
        result.push(c as char);
        n /= radix;
    }
    result.reverse();
    result.into_iter().collect()
}

// ============================================================================
// Void Elements
// ============================================================================

static VOID_ELEMENT_NAMES: phf::Set<&'static str> = phf_set! {
    "area", "base", "br", "col", "command", "embed", "hr", "img", "input",
    "keygen", "link", "meta", "param", "source", "track", "wbr"
};

/// Returns `true` if `name` is of a void element.
pub fn is_void(name: &str) -> bool {
    VOID_ELEMENT_NAMES.contains(name) || name.eq_ignore_ascii_case("!doctype")
}

// ============================================================================
// Reserved Words
// ============================================================================

static RESERVED_WORDS: phf::Set<&'static str> = phf_set! {
    "arguments", "await", "break", "case", "catch", "class", "const", "continue",
    "debugger", "default", "delete", "do", "else", "enum", "eval", "export",
    "extends", "false", "finally", "for", "function", "if", "implements",
    "import", "in", "instanceof", "interface", "let", "new", "null", "package",
    "private", "protected", "public", "return", "static", "super", "switch",
    "this", "throw", "true", "try", "typeof", "var", "void", "while", "with", "yield"
};

/// Returns `true` if `word` is a reserved JavaScript keyword.
pub fn is_reserved(word: &str) -> bool {
    RESERVED_WORDS.contains(word)
}

// ============================================================================
// Event Handling
// ============================================================================

/// Returns `true` if the event name is a capture event.
pub fn is_capture_event(name: &str) -> bool {
    name.ends_with("capture") && name != "gotpointercapture" && name != "lostpointercapture"
}

static DELEGATED_EVENTS: phf::Set<&'static str> = phf_set! {
    "beforeinput", "click", "change", "dblclick", "contextmenu", "focusin",
    "focusout", "input", "keydown", "keyup", "mousedown", "mousemove",
    "mouseout", "mouseover", "mouseup", "pointerdown", "pointermove",
    "pointerout", "pointerover", "pointerup", "touchend", "touchmove", "touchstart"
};

/// Returns `true` if `event_name` is a delegated event.
pub fn can_delegate_event(event_name: &str) -> bool {
    DELEGATED_EVENTS.contains(event_name)
}

static PASSIVE_EVENTS: phf::Set<&'static str> = phf_set! {
    "touchstart", "touchmove"
};

/// Returns `true` if `name` is a passive event.
pub fn is_passive_event(name: &str) -> bool {
    PASSIVE_EVENTS.contains(name)
}

// ============================================================================
// DOM Attributes and Properties
// ============================================================================

static DOM_BOOLEAN_ATTRIBUTES: phf::Set<&'static str> = phf_set! {
    "allowfullscreen", "async", "autofocus", "autoplay", "checked", "controls",
    "default", "disabled", "formnovalidate", "indeterminate", "inert", "ismap",
    "loop", "multiple", "muted", "nomodule", "novalidate", "open", "playsinline",
    "readonly", "required", "reversed", "seamless", "selected", "webkitdirectory",
    "defer", "disablepictureinpicture", "disableremoteplayback"
};

/// Returns `true` if `name` is a boolean attribute.
pub fn is_boolean_attribute(name: &str) -> bool {
    DOM_BOOLEAN_ATTRIBUTES.contains(name)
}

static ATTRIBUTE_ALIASES: phf::Map<&'static str, &'static str> = phf::phf_map! {
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
    "disableremoteplayback" => "disableRemotePlayback"
};

/// Normalizes an attribute name, returning the property name alias if one exists.
/// Note: This performs lowercasing internally for lookup but returns &'static str for aliases.
pub fn normalize_attribute(name: &str) -> &str {
    let lower = name.to_lowercase();
    ATTRIBUTE_ALIASES.get(lower.as_str()).copied().unwrap_or(name)
}

static DOM_PROPERTIES: phf::Set<&'static str> = phf_set! {
    // Boolean attributes
    "allowfullscreen", "async", "autofocus", "autoplay", "checked", "controls",
    "default", "disabled", "formnovalidate", "indeterminate", "inert", "ismap",
    "loop", "multiple", "muted", "nomodule", "novalidate", "open", "playsinline",
    "readonly", "required", "reversed", "seamless", "selected", "webkitdirectory",
    "defer", "disablepictureinpicture", "disableremoteplayback",
    // Additional properties
    "formNoValidate", "isMap", "noModule", "playsInline", "readOnly", "value",
    "volume", "defaultValue", "defaultChecked", "srcObject", "noValidate",
    "allowFullscreen", "disablePictureInPicture", "disableRemotePlayback"
};

/// Returns `true` if `name` is a DOM property.
pub fn is_dom_property(name: &str) -> bool {
    DOM_PROPERTIES.contains(name)
}

static NON_STATIC_PROPERTIES: phf::Set<&'static str> = phf_set! {
    "autofocus", "muted", "defaultValue", "defaultChecked"
};

/// Returns `true` if the given attribute cannot be set through the template string.
pub fn cannot_be_set_statically(name: &str) -> bool {
    NON_STATIC_PROPERTIES.contains(name)
}

// ============================================================================
// Content Editable
// ============================================================================

static CONTENT_EDITABLE_BINDINGS: phf::Set<&'static str> = phf_set! {
    "textContent", "innerHTML", "innerText"
};

/// Returns `true` if `name` is a content editable binding.
pub fn is_content_editable_binding(name: &str) -> bool {
    CONTENT_EDITABLE_BINDINGS.contains(name)
}

// ============================================================================
// Load/Error Elements
// ============================================================================

static LOAD_ERROR_ELEMENTS: phf::Set<&'static str> = phf_set! {
    "body", "embed", "iframe", "img", "link", "object", "script", "style", "track"
};

/// Returns `true` if the element emits `load` and `error` events.
pub fn is_load_error_element(name: &str) -> bool {
    LOAD_ERROR_ELEMENTS.contains(name)
}

// ============================================================================
// SVG Elements
// ============================================================================

static SVG_ELEMENTS: phf::Set<&'static str> = phf_set! {
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
    "meshpatch", "meshrow", "metadata", "missing-glyph", "mpath", "path",
    "pattern", "polygon", "polyline", "radialGradient", "rect", "set",
    "solidcolor", "stop", "svg", "switch", "symbol", "text", "textPath", "tref",
    "tspan", "unknown", "use", "view", "vkern"
};

/// Returns `true` if `name` is an SVG element.
pub fn is_svg(name: &str) -> bool {
    SVG_ELEMENTS.contains(name)
}

// ============================================================================
// MathML Elements
// ============================================================================

static MATHML_ELEMENTS: phf::Set<&'static str> = phf_set! {
    "annotation", "annotation-xml", "maction", "math", "merror", "mfrac", "mi",
    "mmultiscripts", "mn", "mo", "mover", "mpadded", "mphantom", "mprescripts",
    "mroot", "mrow", "ms", "mspace", "msqrt", "mstyle", "msub", "msubsup",
    "msup", "mtable", "mtd", "mtext", "mtr", "munder", "munderover", "semantics"
};

/// Returns `true` if `name` is a MathML element.
pub fn is_mathml(name: &str) -> bool {
    MATHML_ELEMENTS.contains(name)
}

// ============================================================================
// Runes
// ============================================================================

static STATE_CREATION_RUNES: phf::Set<&'static str> = phf_set! {
    "$state", "$state.raw", "$derived", "$derived.by"
};

static RUNES: phf::Set<&'static str> = phf_set! {
    "$state", "$state.raw", "$state.eager", "$state.snapshot", "$derived",
    "$derived.by", "$props", "$props.id", "$bindable", "$effect", "$effect.pre",
    "$effect.tracking", "$effect.root", "$effect.pending", "$inspect",
    "$inspect().with", "$inspect.trace", "$host"
};

/// Returns `true` if `name` is a Svelte rune.
pub fn is_rune(name: &str) -> bool {
    RUNES.contains(name)
}

/// Returns `true` if `name` is a state creation rune ($state, $state.raw, $derived, $derived.by).
pub fn is_state_creation_rune(name: &str) -> bool {
    STATE_CREATION_RUNES.contains(name)
}

// ============================================================================
// Raw Text Elements
// ============================================================================

static RAW_TEXT_ELEMENTS: phf::Set<&'static str> = phf_set! {
    "textarea", "script", "style", "title"
};

/// Returns `true` if `name` is a raw text element.
pub fn is_raw_text_element(name: &str) -> bool {
    RAW_TEXT_ELEMENTS.contains(name)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        // Test basic hashing
        let h = hash("test");
        assert!(!h.is_empty());

        // Same input should produce same output
        assert_eq!(hash("hello"), hash("hello"));

        // Different input should produce different output
        assert_ne!(hash("hello"), hash("world"));

        // Carriage returns should be stripped
        assert_eq!(hash("hello\r\nworld"), hash("hello\nworld"));
    }

    #[test]
    fn test_is_reserved() {
        assert!(is_reserved("await"));
        assert!(is_reserved("class"));
        assert!(is_reserved("function"));
        assert!(is_reserved("return"));
        assert!(!is_reserved("foo"));
        assert!(!is_reserved("myVariable"));
    }

    #[test]
    fn test_is_void() {
        assert!(is_void("br"));
        assert!(is_void("img"));
        assert!(is_void("input"));
        assert!(is_void("!DOCTYPE"));
        assert!(!is_void("div"));
        assert!(!is_void("span"));
    }

    #[test]
    fn test_is_rune() {
        assert!(is_rune("$state"));
        assert!(is_rune("$derived"));
        assert!(is_rune("$effect"));
        assert!(is_rune("$props"));
        assert!(!is_rune("$foo"));
        assert!(!is_rune("state"));
    }

    #[test]
    fn test_is_svg() {
        assert!(is_svg("svg"));
        assert!(is_svg("path"));
        assert!(is_svg("circle"));
        assert!(!is_svg("div"));
    }

    #[test]
    fn test_is_mathml() {
        assert!(is_mathml("math"));
        assert!(is_mathml("mrow"));
        assert!(!is_mathml("div"));
    }
}
