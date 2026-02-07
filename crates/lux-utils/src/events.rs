/// DOM event handling (delegation, passive, capture).
///
/// Reference: `utils.js` lines 109-269

use phf::phf_set;

/// Events that can be delegated to the document root (performance optimization).
pub static DELEGATED_EVENTS: phf::Set<&str> = phf_set! {
    "beforeinput", "click", "change", "dblclick", "contextmenu",
    "focusin", "focusout", "input", "keydown", "keyup",
    "mousedown", "mousemove", "mouseout", "mouseover", "mouseup",
    "pointerdown", "pointermove", "pointerout", "pointerover", "pointerup",
    "touchend", "touchmove", "touchstart",
};

pub fn can_delegate_event(name: &str) -> bool {
    DELEGATED_EVENTS.contains(name)
}

/// Events that should use `{ passive: true }` by default.
pub static PASSIVE_EVENTS: phf::Set<&str> = phf_set! {
    "touchstart", "touchmove",
};

pub fn is_passive_event(name: &str) -> bool {
    PASSIVE_EVENTS.contains(name)
}

/// Check if event name is a capture variant (e.g., `clickcapture`).
///
/// Excludes `gotpointercapture` and `lostpointercapture` which are real events.
pub fn is_capture_event(name: &str) -> bool {
    name.ends_with("capture")
        && name != "gotpointercapture"
        && name != "lostpointercapture"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegated_events() {
        assert!(can_delegate_event("click"));
        assert!(can_delegate_event("input"));
        assert!(can_delegate_event("mousedown"));
        assert!(can_delegate_event("pointerup"));
        assert!(!can_delegate_event("scroll"));
        assert!(!can_delegate_event("resize"));
        assert!(!can_delegate_event("load"));
    }

    #[test]
    fn test_passive_events() {
        assert!(is_passive_event("touchstart"));
        assert!(is_passive_event("touchmove"));
        assert!(!is_passive_event("touchend"));
        assert!(!is_passive_event("click"));
    }

    #[test]
    fn test_capture_events() {
        assert!(is_capture_event("clickcapture"));
        assert!(is_capture_event("focuscapture"));
        assert!(!is_capture_event("gotpointercapture"));
        assert!(!is_capture_event("lostpointercapture"));
        assert!(!is_capture_event("click"));
    }
}
