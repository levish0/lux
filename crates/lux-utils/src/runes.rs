//! Rune utilities.

/// All Svelte runes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rune {
    // $state family
    State,
    StateRaw,
    StateEager,
    StateSnapshot,

    // $derived family
    Derived,
    DerivedBy,

    // $props family
    Props,
    PropsId,
    Bindable,

    // $effect family
    Effect,
    EffectPre,
    EffectTracking,
    EffectRoot,
    EffectPending,

    // $inspect family
    Inspect,
    InspectWith,
    InspectTrace,

    // $host
    Host,
}

impl Rune {
    /// Returns the string representation of this rune.
    pub fn as_str(&self) -> &'static str {
        match self {
            Rune::State => "$state",
            Rune::StateRaw => "$state.raw",
            Rune::StateEager => "$state.eager",
            Rune::StateSnapshot => "$state.snapshot",
            Rune::Derived => "$derived",
            Rune::DerivedBy => "$derived.by",
            Rune::Props => "$props",
            Rune::PropsId => "$props.id",
            Rune::Bindable => "$bindable",
            Rune::Effect => "$effect",
            Rune::EffectPre => "$effect.pre",
            Rune::EffectTracking => "$effect.tracking",
            Rune::EffectRoot => "$effect.root",
            Rune::EffectPending => "$effect.pending",
            Rune::Inspect => "$inspect",
            Rune::InspectWith => "$inspect().with",
            Rune::InspectTrace => "$inspect.trace",
            Rune::Host => "$host",
        }
    }

    /// Returns true if this is a state creation rune.
    pub fn is_state_creation(&self) -> bool {
        matches!(
            self,
            Rune::State | Rune::StateRaw | Rune::Derived | Rune::DerivedBy
        )
    }
}

impl std::fmt::Display for Rune {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Parses a string into a Rune, if it matches.
pub fn get_rune(name: &str) -> Option<Rune> {
    match name {
        "$state" => Some(Rune::State),
        "$state.raw" => Some(Rune::StateRaw),
        "$state.eager" => Some(Rune::StateEager),
        "$state.snapshot" => Some(Rune::StateSnapshot),
        "$derived" => Some(Rune::Derived),
        "$derived.by" => Some(Rune::DerivedBy),
        "$props" => Some(Rune::Props),
        "$props.id" => Some(Rune::PropsId),
        "$bindable" => Some(Rune::Bindable),
        "$effect" => Some(Rune::Effect),
        "$effect.pre" => Some(Rune::EffectPre),
        "$effect.tracking" => Some(Rune::EffectTracking),
        "$effect.root" => Some(Rune::EffectRoot),
        "$effect.pending" => Some(Rune::EffectPending),
        "$inspect" => Some(Rune::Inspect),
        "$inspect().with" => Some(Rune::InspectWith),
        "$inspect.trace" => Some(Rune::InspectTrace),
        "$host" => Some(Rune::Host),
        _ => None,
    }
}

/// Returns `true` if `name` is a Svelte rune.
pub fn is_rune(name: &str) -> bool {
    get_rune(name).is_some()
}

/// Returns `true` if `name` is a state creation rune ($state, $state.raw, $derived, $derived.by).
pub fn is_state_creation_rune(name: &str) -> bool {
    get_rune(name).map_or(false, |r| r.is_state_creation())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_get_rune() {
        assert_eq!(get_rune("$state"), Some(Rune::State));
        assert_eq!(get_rune("$state.raw"), Some(Rune::StateRaw));
        assert_eq!(get_rune("$derived"), Some(Rune::Derived));
        assert_eq!(get_rune("$derived.by"), Some(Rune::DerivedBy));
        assert_eq!(get_rune("$effect"), Some(Rune::Effect));
        assert_eq!(get_rune("$effect.pre"), Some(Rune::EffectPre));
        assert_eq!(get_rune("$props"), Some(Rune::Props));
        assert_eq!(get_rune("$bindable"), Some(Rune::Bindable));
        assert_eq!(get_rune("$inspect"), Some(Rune::Inspect));
        assert_eq!(get_rune("$host"), Some(Rune::Host));
        assert_eq!(get_rune("$foo"), None);
        assert_eq!(get_rune("state"), None);
    }

    #[test]
    fn test_rune_as_str() {
        assert_eq!(Rune::State.as_str(), "$state");
        assert_eq!(Rune::StateRaw.as_str(), "$state.raw");
        assert_eq!(Rune::DerivedBy.as_str(), "$derived.by");
        assert_eq!(Rune::EffectTracking.as_str(), "$effect.tracking");
        assert_eq!(Rune::InspectWith.as_str(), "$inspect().with");
    }

    #[test]
    fn test_rune_is_state_creation() {
        assert!(Rune::State.is_state_creation());
        assert!(Rune::StateRaw.is_state_creation());
        assert!(Rune::Derived.is_state_creation());
        assert!(Rune::DerivedBy.is_state_creation());
        assert!(!Rune::Effect.is_state_creation());
        assert!(!Rune::Props.is_state_creation());
    }
}
