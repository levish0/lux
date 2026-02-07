use strsim::jaro_winkler;

/// Fuzzy string matching for error suggestions ("Did you mean ...?").
///
/// Uses `strsim::jaro_winkler` for string similarity — handles transpositions
/// well and gives prefix-matching bonus, ideal for typo correction.
/// This intentionally differs from Svelte's custom n-gram implementation
/// since it's only used for error messages, not compiled output.

/// Find the best fuzzy match for `name` among `candidates`.
///
/// Returns `Some(match)` if the best match has Jaro-Winkler
/// similarity >= 0.8, `None` otherwise.
pub fn fuzzymatch<'a>(name: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let threshold = 0.8;
    let mut best_score = 0.0f64;
    let mut best_match = None;

    for &candidate in candidates {
        let score = jaro_winkler(name, candidate);
        if score > best_score {
            best_score = score;
            best_match = Some(candidate);
        }
    }

    if best_score >= threshold {
        best_match
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_close_match() {
        let candidates = ["onclick", "onchange", "oninput", "onsubmit"];
        assert_eq!(fuzzymatch("onlick", &candidates), Some("onclick"));
    }

    #[test]
    fn test_exact_match() {
        let candidates = ["foo", "bar", "baz"];
        assert_eq!(fuzzymatch("foo", &candidates), Some("foo"));
    }

    #[test]
    fn test_no_match() {
        let candidates = ["onclick", "onchange", "oninput"];
        assert_eq!(fuzzymatch("zzzzzzzzz", &candidates), None);
    }

    #[test]
    fn test_empty_candidates() {
        let candidates: &[&str] = &[];
        assert_eq!(fuzzymatch("foo", candidates), None);
    }

    #[test]
    fn test_typo_correction() {
        let candidates = ["class", "style", "onclick", "disabled"];
        assert_eq!(fuzzymatch("clas", &candidates), Some("class"));
        assert_eq!(fuzzymatch("styel", &candidates), Some("style"));
    }
}
