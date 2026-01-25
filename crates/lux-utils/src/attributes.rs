//! Attribute-related utilities.

use phf::phf_set;

// ============================================================================
// DOM Boolean Attributes
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

// ============================================================================
// Attribute Aliases
// ============================================================================

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
pub fn normalize_attribute(name: &str) -> &str {
    let lower = name.to_lowercase();
    ATTRIBUTE_ALIASES.get(lower.as_str()).copied().unwrap_or(name)
}

// ============================================================================
// DOM Properties
// ============================================================================

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

// ============================================================================
// Non-static Properties
// ============================================================================

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
