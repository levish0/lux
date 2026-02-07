/// Bindable properties per element type.
///
/// Reference: `compiler/phases/bindings.js` lines 14-227

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingProperty {
    /// DOM event that triggers update (e.g., "change", "input", "timeupdate").
    pub event: Option<&'static str>,
    /// Whether the binding is bidirectional (can update DOM property).
    pub bidirectional: bool,
    /// Whether to exclude from SSR output.
    pub omit_in_ssr: bool,
    /// Elements this binding is valid on (empty slice = any element).
    pub valid_elements: &'static [&'static str],
}

/// Look up binding property metadata by binding name.
pub fn get_binding_property(name: &str) -> Option<BindingProperty> {
    match name {
        // --- Media bindings (audio, video) ---
        "currentTime" => Some(BindingProperty {
            event: Some("timeupdate"),
            bidirectional: true,
            omit_in_ssr: true,
            valid_elements: &["audio", "video"],
        }),
        "duration" => Some(BindingProperty {
            event: Some("durationchange"),
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &["audio", "video"],
        }),
        "focused" => Some(BindingProperty {
            event: None,
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &["audio", "video"],
        }),
        "paused" => Some(BindingProperty {
            event: None,
            bidirectional: true,
            omit_in_ssr: true,
            valid_elements: &["audio", "video"],
        }),
        "volume" => Some(BindingProperty {
            event: Some("volumechange"),
            bidirectional: true,
            omit_in_ssr: true,
            valid_elements: &["audio", "video"],
        }),
        "muted" => Some(BindingProperty {
            event: Some("volumechange"),
            bidirectional: true,
            omit_in_ssr: false,
            valid_elements: &["audio", "video"],
        }),
        "playbackRate" => Some(BindingProperty {
            event: Some("ratechange"),
            bidirectional: true,
            omit_in_ssr: true,
            valid_elements: &["audio", "video"],
        }),
        "seeking" => Some(BindingProperty {
            event: None,
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &["audio", "video"],
        }),
        "ended" => Some(BindingProperty {
            event: None,
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &["audio", "video"],
        }),
        "readyState" => Some(BindingProperty {
            event: None,
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &["audio", "video"],
        }),
        "buffered" => Some(BindingProperty {
            event: None,
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &["audio", "video"],
        }),
        "seekable" => Some(BindingProperty {
            event: None,
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &["audio", "video"],
        }),
        "played" => Some(BindingProperty {
            event: None,
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &["audio", "video"],
        }),

        // --- Video-specific ---
        "videoHeight" => Some(BindingProperty {
            event: Some("resize"),
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &["video"],
        }),
        "videoWidth" => Some(BindingProperty {
            event: Some("resize"),
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &["video"],
        }),

        // --- Image ---
        "naturalWidth" => Some(BindingProperty {
            event: Some("load"),
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &["img"],
        }),
        "naturalHeight" => Some(BindingProperty {
            event: Some("load"),
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &["img"],
        }),

        // --- Form ---
        "value" => Some(BindingProperty {
            event: None,
            bidirectional: true,
            omit_in_ssr: false,
            valid_elements: &["input", "textarea", "select"],
        }),
        "checked" => Some(BindingProperty {
            event: None,
            bidirectional: true,
            omit_in_ssr: false,
            valid_elements: &["input"],
        }),
        "indeterminate" => Some(BindingProperty {
            event: Some("change"),
            bidirectional: true,
            omit_in_ssr: true,
            valid_elements: &["input"],
        }),
        "group" => Some(BindingProperty {
            event: None,
            bidirectional: true,
            omit_in_ssr: false,
            valid_elements: &["input"],
        }),
        "files" => Some(BindingProperty {
            event: Some("change"),
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &["input"],
        }),

        // --- Details ---
        "open" => Some(BindingProperty {
            event: Some("toggle"),
            bidirectional: true,
            omit_in_ssr: false,
            valid_elements: &["details"],
        }),

        // --- Dimensions (any element) ---
        "clientWidth" | "clientHeight" | "offsetWidth" | "offsetHeight"
        | "contentRect" | "contentBoxSize" | "borderBoxSize"
        | "devicePixelContentBoxSize" => Some(BindingProperty {
            event: None,
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &[],
        }),

        // --- Content editable (any element) ---
        "innerText" | "innerHTML" | "textContent" => Some(BindingProperty {
            event: None,
            bidirectional: true,
            omit_in_ssr: true,
            valid_elements: &[],
        }),

        // --- Window bindings (svelte:window) ---
        "innerWidth" | "innerHeight" | "outerWidth" | "outerHeight" => Some(BindingProperty {
            event: None,
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &[],
        }),
        "scrollX" | "scrollY" => Some(BindingProperty {
            event: None,
            bidirectional: true,
            omit_in_ssr: true,
            valid_elements: &[],
        }),
        "online" => Some(BindingProperty {
            event: None,
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &[],
        }),
        "devicePixelRatio" => Some(BindingProperty {
            event: None,
            bidirectional: false,
            omit_in_ssr: true,
            valid_elements: &[],
        }),

        // --- Document bindings (svelte:document) ---
        "activeElement" | "fullscreenElement" | "pointerLockElement" | "visibilityState" => {
            Some(BindingProperty {
                event: None,
                bidirectional: false,
                omit_in_ssr: true,
                valid_elements: &[],
            })
        }

        _ => None,
    }
}

/// Check if a binding name is a known binding.
pub fn is_known_binding(name: &str) -> bool {
    get_binding_property(name).is_some()
}

/// Check if a binding is valid for a given element.
pub fn is_binding_valid_for_element(binding: &str, element: &str) -> bool {
    match get_binding_property(binding) {
        Some(prop) => prop.valid_elements.is_empty() || prop.valid_elements.contains(&element),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_bindings() {
        let prop = get_binding_property("currentTime").unwrap();
        assert_eq!(prop.event, Some("timeupdate"));
        assert!(prop.bidirectional);
        assert!(prop.omit_in_ssr);
        assert!(prop.valid_elements.contains(&"audio"));
        assert!(prop.valid_elements.contains(&"video"));
    }

    #[test]
    fn test_form_bindings() {
        let prop = get_binding_property("value").unwrap();
        assert!(prop.bidirectional);
        assert!(!prop.omit_in_ssr);
        assert!(prop.valid_elements.contains(&"input"));
        assert!(prop.valid_elements.contains(&"textarea"));
        assert!(prop.valid_elements.contains(&"select"));
    }

    #[test]
    fn test_dimension_bindings() {
        let prop = get_binding_property("clientWidth").unwrap();
        assert!(!prop.bidirectional);
        assert!(prop.omit_in_ssr);
        assert!(prop.valid_elements.is_empty()); // any element
    }

    #[test]
    fn test_unknown_binding() {
        assert!(get_binding_property("nonexistent").is_none());
        assert!(!is_known_binding("nonexistent"));
    }

    #[test]
    fn test_binding_valid_for_element() {
        assert!(is_binding_valid_for_element("value", "input"));
        assert!(is_binding_valid_for_element("value", "textarea"));
        assert!(!is_binding_valid_for_element("value", "div"));
        assert!(is_binding_valid_for_element("clientWidth", "div")); // any element
        assert!(is_binding_valid_for_element("clientWidth", "span")); // any element
    }
}
