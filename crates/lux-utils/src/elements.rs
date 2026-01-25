//! Element-related utilities.

use phf::phf_set;

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
// Load/Error Elements
// ============================================================================

static LOAD_ERROR_ELEMENTS: phf::Set<&'static str> = phf_set! {
    "body", "embed", "iframe", "img", "link", "object", "script", "style", "track"
};

/// Returns `true` if the element emits `load` and `error` events.
pub fn is_load_error_element(name: &str) -> bool {
    LOAD_ERROR_ELEMENTS.contains(name)
}

#[cfg(test)]
mod tests {
    use super::*;

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
