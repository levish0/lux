/// Element classification (void, SVG, MathML, etc.).
///
/// Reference: `utils.js` lines 16-428

use phf::phf_set;

pub static VOID_ELEMENTS: phf::Set<&str> = phf_set! {
    "area", "base", "br", "col", "command", "embed", "hr", "img",
    "input", "keygen", "link", "meta", "param", "source", "track", "wbr",
};

pub fn is_void(name: &str) -> bool {
    VOID_ELEMENTS.contains(name) || name.eq_ignore_ascii_case("!doctype")
}

pub static SVG_ELEMENTS: phf::Set<&str> = phf_set! {
    "altGlyph", "altGlyphDef", "altGlyphItem", "animate", "animateColor",
    "animateMotion", "animateTransform", "circle", "clipPath", "color-profile",
    "cursor", "defs", "desc", "ellipse", "feBlend", "feColorMatrix",
    "feComponentTransfer", "feComposite", "feConvolveMatrix",
    "feDiffuseLighting", "feDisplacementMap", "feDistantLight", "feDropShadow",
    "feFlood", "feFuncA", "feFuncB", "feFuncG", "feFuncR",
    "feGaussianBlur", "feImage", "feMerge", "feMergeNode", "feMorphology",
    "feOffset", "fePointLight", "feSpecularLighting", "feSpotLight",
    "feTile", "feTurbulence", "filter", "font", "font-face",
    "font-face-format", "font-face-name", "font-face-src", "font-face-uri",
    "foreignObject", "g", "glyph", "glyphRef", "hatch", "hatchpath",
    "hkern", "image", "line", "linearGradient", "marker", "mask",
    "metadata", "missing-glyph", "mpath", "path", "pattern", "polygon",
    "polyline", "radialGradient", "rect", "set", "stop", "svg", "switch",
    "symbol", "text", "textPath", "tref", "tspan", "use", "view", "vkern",
};

pub fn is_svg(name: &str) -> bool {
    SVG_ELEMENTS.contains(name)
}

pub static MATHML_ELEMENTS: phf::Set<&str> = phf_set! {
    "annotation", "maction", "math", "merror", "mfrac", "mi",
    "mmultiscripts", "mn", "mo", "mover", "mpadded", "mphantom",
    "mprescripts", "mroot", "mrow", "ms", "mspace", "msqrt", "mstyle",
    "msub", "msubsup", "msup", "mtable", "mtd", "mtext", "mtr",
    "munder", "munderover", "semantics",
};

pub fn is_mathml(name: &str) -> bool {
    MATHML_ELEMENTS.contains(name)
}

pub static RAW_TEXT_ELEMENTS: phf::Set<&str> = phf_set! {
    "textarea", "script", "style", "title",
};

pub fn is_raw_text_element(name: &str) -> bool {
    RAW_TEXT_ELEMENTS.contains(name)
}

pub static LOAD_ERROR_ELEMENTS: phf::Set<&str> = phf_set! {
    "body", "embed", "iframe", "img", "link", "object", "script", "style", "track",
};

pub fn is_load_error_element(name: &str) -> bool {
    LOAD_ERROR_ELEMENTS.contains(name)
}

pub static INTERACTIVE_ELEMENTS: phf::Set<&str> = phf_set! {
    "a", "button", "details", "embed", "iframe", "label", "select", "textarea",
};

pub fn is_interactive_element(name: &str) -> bool {
    INTERACTIVE_ELEMENTS.contains(name)
}

pub static LABELABLE_ELEMENTS: phf::Set<&str> = phf_set! {
    "button", "input", "keygen", "meter", "output", "progress", "select", "textarea",
};

pub fn is_labelable_element(name: &str) -> bool {
    LABELABLE_ELEMENTS.contains(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_void_elements() {
        assert!(is_void("br"));
        assert!(is_void("hr"));
        assert!(is_void("img"));
        assert!(is_void("input"));
        assert!(is_void("!DOCTYPE"));
        assert!(!is_void("div"));
        assert!(!is_void("span"));
    }

    #[test]
    fn test_svg_elements() {
        assert!(is_svg("svg"));
        assert!(is_svg("path"));
        assert!(is_svg("circle"));
        assert!(is_svg("foreignObject"));
        assert!(is_svg("feGaussianBlur"));
        assert!(!is_svg("div"));
    }

    #[test]
    fn test_mathml_elements() {
        assert!(is_mathml("math"));
        assert!(is_mathml("mrow"));
        assert!(is_mathml("msqrt"));
        assert!(!is_mathml("div"));
    }

    #[test]
    fn test_raw_text_elements() {
        assert!(is_raw_text_element("script"));
        assert!(is_raw_text_element("style"));
        assert!(is_raw_text_element("textarea"));
        assert!(is_raw_text_element("title"));
        assert!(!is_raw_text_element("div"));
    }
}
