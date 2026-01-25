//! Fuzzy string matching utilities.
//!
//! Uses the strsim crate for string similarity calculations.

use strsim::jaro_winkler;

/// Finds the best fuzzy match for a name from a list of candidates.
/// Returns the match if the similarity score is above 0.7.
pub fn fuzzymatch<'a>(name: &str, candidates: &[&'a str]) -> Option<&'a str> {
    if candidates.is_empty() {
        return None;
    }

    let name_lower = name.to_lowercase();

    // First check for exact match (case insensitive)
    for &candidate in candidates {
        if candidate.to_lowercase() == name_lower {
            return Some(candidate);
        }
    }

    // Find the best fuzzy match
    let mut best_match: Option<&str> = None;
    let mut best_score: f64 = 0.0;

    for &candidate in candidates {
        let score = jaro_winkler(&name_lower, &candidate.to_lowercase());
        if score > best_score {
            best_score = score;
            best_match = Some(candidate);
        }
    }

    // Only return if score is above threshold (0.7)
    if best_score > 0.7 {
        best_match
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzymatch_exact() {
        let candidates = ["value", "checked", "group"];
        assert_eq!(fuzzymatch("value", &candidates), Some("value"));
        assert_eq!(fuzzymatch("VALUE", &candidates), Some("value"));
    }

    #[test]
    fn test_fuzzymatch_typo() {
        let candidates = ["value", "checked", "group", "clientWidth"];
        // Common typos
        assert_eq!(fuzzymatch("valeu", &candidates), Some("value"));
        assert_eq!(fuzzymatch("clienWidth", &candidates), Some("clientWidth"));
    }

    #[test]
    fn test_fuzzymatch_no_match() {
        let candidates = ["value", "checked", "group"];
        assert_eq!(fuzzymatch("xyz", &candidates), None);
    }

    #[test]
    fn test_fuzzymatch_empty() {
        let candidates: [&str; 0] = [];
        assert_eq!(fuzzymatch("value", &candidates), None);
    }
}
