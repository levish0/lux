//! Event-related utilities.

use phf::phf_set;

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
