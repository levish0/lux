/// Attribute properties (boolean, DOM props, aliases).
///
/// Reference: `utils.js` lines 147-260

use phf::{phf_map, phf_set};

pub static BOOLEAN_ATTRIBUTES: phf::Set<&str> = phf_set! {
    "allowfullscreen", "async", "autofocus", "autoplay", "checked",
    "controls", "default", "defer", "disabled", "disablepictureinpicture",
    "disableremoteplayback", "formnovalidate", "indeterminate", "inert",
    "ismap", "loop", "multiple", "muted", "nomodule", "novalidate",
    "open", "playsinline", "readonly", "required", "reversed",
    "seamless", "selected", "webkitdirectory",
};

pub fn is_boolean_attribute(name: &str) -> bool {
    BOOLEAN_ATTRIBUTES.contains(name)
}

/// Attribute names that map to different DOM property names.
pub static ATTRIBUTE_ALIASES: phf::Map<&str, &str> = phf_map! {
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
};

pub fn normalize_attribute(name: &str) -> &str {
    ATTRIBUTE_ALIASES.get(name).copied().unwrap_or(name)
}

/// Attributes that have corresponding JavaScript DOM properties.
pub fn is_dom_property(name: &str) -> bool {
    BOOLEAN_ATTRIBUTES.contains(name)
        || matches!(
            name,
            "value" | "volume" | "defaultValue" | "defaultChecked" | "srcObject"
        )
}

/// Attributes that cannot be set statically in HTML — require JS to set.
pub static NON_STATIC_PROPERTIES: phf::Set<&str> = phf_set! {
    "autofocus", "muted", "defaultValue", "defaultChecked",
};

pub fn cannot_be_set_statically(name: &str) -> bool {
    NON_STATIC_PROPERTIES.contains(name)
}

pub static CONTENT_EDITABLE_BINDINGS: phf::Set<&str> = phf_set! {
    "textContent", "innerHTML", "innerText",
};

pub fn is_content_editable_binding(name: &str) -> bool {
    CONTENT_EDITABLE_BINDINGS.contains(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_attributes() {
        assert!(is_boolean_attribute("checked"));
        assert!(is_boolean_attribute("disabled"));
        assert!(is_boolean_attribute("readonly"));
        assert!(!is_boolean_attribute("value"));
        assert!(!is_boolean_attribute("class"));
    }

    #[test]
    fn test_attribute_aliases() {
        assert_eq!(normalize_attribute("readonly"), "readOnly");
        assert_eq!(normalize_attribute("ismap"), "isMap");
        assert_eq!(normalize_attribute("class"), "class"); // unchanged
    }

    #[test]
    fn test_dom_property() {
        assert!(is_dom_property("checked"));
        assert!(is_dom_property("value"));
        assert!(is_dom_property("volume"));
        assert!(!is_dom_property("class"));
        assert!(!is_dom_property("style"));
    }

    #[test]
    fn test_non_static() {
        assert!(cannot_be_set_statically("autofocus"));
        assert!(cannot_be_set_statically("muted"));
        assert!(!cannot_be_set_statically("checked"));
    }
}
