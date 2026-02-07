/// Svelte 5 rune definitions.
///
/// Reference: `utils.js` lines 430-473
use phf::phf_set;

pub static RUNES: phf::Set<&str> = phf_set! {
    "$state", "$state.raw",
    "$derived", "$derived.by",
    "$effect", "$effect.pre", "$effect.tracking", "$effect.root", "$effect.pending",
    "$props", "$props.id",
    "$bindable",
    "$inspect", "$inspect.trace",
    "$host",
};

/// Runes that create reactive state (used for BindingKind classification).
pub static STATE_CREATION_RUNES: phf::Set<&str> = phf_set! {
    "$state", "$state.raw", "$derived", "$derived.by",
};

pub fn is_rune(name: &str) -> bool {
    RUNES.contains(name)
}

pub fn is_state_creation_rune(name: &str) -> bool {
    STATE_CREATION_RUNES.contains(name)
}

/// Check if a name could be a rune prefix (starts with `$` and has a known base).
///
/// Used to detect potential rune usage like `$state`, `$effect.pre`, etc.
pub fn get_rune(name: &str) -> Option<&'static str> {
    // Try exact match first
    if RUNES.contains(name) {
        // Return the static reference from the set
        return RUNES.iter().find(|&r| r == &name).copied();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runes() {
        assert!(is_rune("$state"));
        assert!(is_rune("$state.raw"));
        assert!(is_rune("$derived"));
        assert!(is_rune("$derived.by"));
        assert!(is_rune("$effect"));
        assert!(is_rune("$effect.pre"));
        assert!(is_rune("$props"));
        assert!(is_rune("$props.id"));
        assert!(is_rune("$bindable"));
        assert!(is_rune("$inspect"));
        assert!(is_rune("$host"));
        assert!(!is_rune("$store"));
        assert!(!is_rune("$foo"));
    }

    #[test]
    fn test_state_creation_runes() {
        assert!(is_state_creation_rune("$state"));
        assert!(is_state_creation_rune("$state.raw"));
        assert!(is_state_creation_rune("$derived"));
        assert!(is_state_creation_rune("$derived.by"));
        assert!(!is_state_creation_rune("$effect"));
        assert!(!is_state_creation_rune("$props"));
    }

    #[test]
    fn test_get_rune() {
        assert_eq!(get_rune("$state"), Some("$state"));
        assert_eq!(get_rune("$effect.pre"), Some("$effect.pre"));
        assert_eq!(get_rune("$unknown"), None);
    }
}
