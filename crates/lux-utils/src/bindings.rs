//! Binding property utilities.

use phf::phf_map;

/// Properties for a binding.
#[derive(Debug, Clone, Default)]
pub struct BindingProperty {
    /// Event that notifies of a change to this property
    pub event: Option<&'static str>,
    /// Whether updates are written to the DOM property
    pub bidirectional: bool,
    /// Whether the binding should be omitted in SSR
    pub omit_in_ssr: bool,
    /// If set, the binding is only valid on these elements
    pub valid_elements: Option<&'static [&'static str]>,
    /// If set, the binding is invalid on these elements
    pub invalid_elements: Option<&'static [&'static str]>,
}

/// Gets the binding property for a given name, if it exists.
pub fn get_binding_property(name: &str) -> Option<&'static BindingProperty> {
    BINDING_PROPERTIES.get(name)
}

/// Returns true if the binding name is a known binding.
pub fn is_known_binding(name: &str) -> bool {
    BINDING_PROPERTIES.contains_key(name)
}

/// All known binding properties.
static BINDING_PROPERTIES: phf::Map<&'static str, BindingProperty> = phf_map! {
    // Media bindings
    "currentTime" => BindingProperty {
        event: None,
        bidirectional: true,
        omit_in_ssr: true,
        valid_elements: Some(&["audio", "video"]),
        invalid_elements: None,
    },
    "duration" => BindingProperty {
        event: Some("durationchange"),
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["audio", "video"]),
        invalid_elements: None,
    },
    "paused" => BindingProperty {
        event: None,
        bidirectional: true,
        omit_in_ssr: true,
        valid_elements: Some(&["audio", "video"]),
        invalid_elements: None,
    },
    "buffered" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["audio", "video"]),
        invalid_elements: None,
    },
    "seekable" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["audio", "video"]),
        invalid_elements: None,
    },
    "played" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["audio", "video"]),
        invalid_elements: None,
    },
    "volume" => BindingProperty {
        event: None,
        bidirectional: true,
        omit_in_ssr: true,
        valid_elements: Some(&["audio", "video"]),
        invalid_elements: None,
    },
    "muted" => BindingProperty {
        event: None,
        bidirectional: true,
        omit_in_ssr: true,
        valid_elements: Some(&["audio", "video"]),
        invalid_elements: None,
    },
    "playbackRate" => BindingProperty {
        event: None,
        bidirectional: true,
        omit_in_ssr: true,
        valid_elements: Some(&["audio", "video"]),
        invalid_elements: None,
    },
    "seeking" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["audio", "video"]),
        invalid_elements: None,
    },
    "ended" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["audio", "video"]),
        invalid_elements: None,
    },
    "readyState" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["audio", "video"]),
        invalid_elements: None,
    },
    // Video bindings
    "videoHeight" => BindingProperty {
        event: Some("resize"),
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["video"]),
        invalid_elements: None,
    },
    "videoWidth" => BindingProperty {
        event: Some("resize"),
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["video"]),
        invalid_elements: None,
    },
    // Image bindings
    "naturalWidth" => BindingProperty {
        event: Some("load"),
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["img"]),
        invalid_elements: None,
    },
    "naturalHeight" => BindingProperty {
        event: Some("load"),
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["img"]),
        invalid_elements: None,
    },
    // Document bindings
    "activeElement" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["svelte:document"]),
        invalid_elements: None,
    },
    "fullscreenElement" => BindingProperty {
        event: Some("fullscreenchange"),
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["svelte:document"]),
        invalid_elements: None,
    },
    "pointerLockElement" => BindingProperty {
        event: Some("pointerlockchange"),
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["svelte:document"]),
        invalid_elements: None,
    },
    "visibilityState" => BindingProperty {
        event: Some("visibilitychange"),
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["svelte:document"]),
        invalid_elements: None,
    },
    // Window bindings
    "innerWidth" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["svelte:window"]),
        invalid_elements: None,
    },
    "innerHeight" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["svelte:window"]),
        invalid_elements: None,
    },
    "outerWidth" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["svelte:window"]),
        invalid_elements: None,
    },
    "outerHeight" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["svelte:window"]),
        invalid_elements: None,
    },
    "scrollX" => BindingProperty {
        event: None,
        bidirectional: true,
        omit_in_ssr: true,
        valid_elements: Some(&["svelte:window"]),
        invalid_elements: None,
    },
    "scrollY" => BindingProperty {
        event: None,
        bidirectional: true,
        omit_in_ssr: true,
        valid_elements: Some(&["svelte:window"]),
        invalid_elements: None,
    },
    "online" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["svelte:window"]),
        invalid_elements: None,
    },
    "devicePixelRatio" => BindingProperty {
        event: Some("resize"),
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: Some(&["svelte:window"]),
        invalid_elements: None,
    },
    // Dimension bindings
    "clientWidth" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: None,
        invalid_elements: Some(&["svelte:window", "svelte:document"]),
    },
    "clientHeight" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: None,
        invalid_elements: Some(&["svelte:window", "svelte:document"]),
    },
    "offsetWidth" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: None,
        invalid_elements: Some(&["svelte:window", "svelte:document"]),
    },
    "offsetHeight" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: None,
        invalid_elements: Some(&["svelte:window", "svelte:document"]),
    },
    "contentRect" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: None,
        invalid_elements: Some(&["svelte:window", "svelte:document"]),
    },
    "contentBoxSize" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: None,
        invalid_elements: Some(&["svelte:window", "svelte:document"]),
    },
    "borderBoxSize" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: None,
        invalid_elements: Some(&["svelte:window", "svelte:document"]),
    },
    "devicePixelContentBoxSize" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: None,
        invalid_elements: Some(&["svelte:window", "svelte:document"]),
    },
    // Input bindings
    "indeterminate" => BindingProperty {
        event: Some("change"),
        bidirectional: true,
        omit_in_ssr: true,
        valid_elements: Some(&["input"]),
        invalid_elements: None,
    },
    "checked" => BindingProperty {
        event: None,
        bidirectional: true,
        omit_in_ssr: false,
        valid_elements: Some(&["input"]),
        invalid_elements: None,
    },
    "group" => BindingProperty {
        event: None,
        bidirectional: true,
        omit_in_ssr: false,
        valid_elements: Some(&["input"]),
        invalid_elements: None,
    },
    "value" => BindingProperty {
        event: None,
        bidirectional: true,
        omit_in_ssr: false,
        valid_elements: Some(&["input", "textarea", "select"]),
        invalid_elements: None,
    },
    "files" => BindingProperty {
        event: None,
        bidirectional: true,
        omit_in_ssr: true,
        valid_elements: Some(&["input"]),
        invalid_elements: None,
    },
    // Various bindings
    "focused" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: false,
        valid_elements: None,
        invalid_elements: None,
    },
    "this" => BindingProperty {
        event: None,
        bidirectional: false,
        omit_in_ssr: true,
        valid_elements: None,
        invalid_elements: None,
    },
    "innerText" => BindingProperty {
        event: None,
        bidirectional: true,
        omit_in_ssr: false,
        valid_elements: None,
        invalid_elements: Some(&["svelte:window", "svelte:document"]),
    },
    "innerHTML" => BindingProperty {
        event: None,
        bidirectional: true,
        omit_in_ssr: false,
        valid_elements: None,
        invalid_elements: Some(&["svelte:window", "svelte:document"]),
    },
    "textContent" => BindingProperty {
        event: None,
        bidirectional: true,
        omit_in_ssr: false,
        valid_elements: None,
        invalid_elements: Some(&["svelte:window", "svelte:document"]),
    },
    "open" => BindingProperty {
        event: Some("toggle"),
        bidirectional: true,
        omit_in_ssr: false,
        valid_elements: Some(&["details"]),
        invalid_elements: None,
    },
};
