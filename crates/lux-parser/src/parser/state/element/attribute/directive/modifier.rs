use lux_ast::template::directive::{EventModifier, StyleModifier, TransitionModifier};

pub(super) fn parse_event_modifier(modifier: &str) -> Option<EventModifier> {
    match modifier {
        "capture" => Some(EventModifier::Capture),
        "nonpassive" => Some(EventModifier::Nonpassive),
        "once" => Some(EventModifier::Once),
        "passive" => Some(EventModifier::Passive),
        "preventDefault" => Some(EventModifier::PreventDefault),
        "self" => Some(EventModifier::Self_),
        "stopImmediatePropagation" => Some(EventModifier::StopImmediatePropagation),
        "stopPropagation" => Some(EventModifier::StopPropagation),
        "trusted" => Some(EventModifier::Trusted),
        _ => None,
    }
}

pub(super) fn parse_style_modifier(modifier: &str) -> Option<StyleModifier> {
    match modifier {
        "important" => Some(StyleModifier::Important),
        _ => None,
    }
}

pub(super) fn parse_transition_modifier(modifier: &str) -> Option<TransitionModifier> {
    match modifier {
        "local" => Some(TransitionModifier::Local),
        "global" => Some(TransitionModifier::Global),
        _ => None,
    }
}
