//! Hash utilities.

/// Computes a hash of a string using DJB2 algorithm, returning base36.
/// Matches Svelte's hash function from utils.js.
pub fn hash(s: &str) -> String {
    // Remove carriage returns like the original
    let s = s.replace('\r', "");
    let mut hash: u32 = 5381;

    // Use bytes for closer matching to charCodeAt (ASCII range)
    for &b in s.as_bytes().iter().rev() {
        hash = ((hash << 5).wrapping_sub(hash)) ^ (b as u32);
    }

    // Convert to base36
    format_radix(hash, 36)
}

/// Formats a number in the given radix (base).
fn format_radix(mut n: u32, radix: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }

    let mut result = Vec::new();
    while n > 0 {
        let digit = (n % radix) as u8;
        let c = if digit < 10 {
            b'0' + digit
        } else {
            b'a' + (digit - 10)
        };
        result.push(c as char);
        n /= radix;
    }
    result.reverse();
    result.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        // Test basic hashing
        let h = hash("test");
        assert!(!h.is_empty());

        // Same input should produce same output
        assert_eq!(hash("hello"), hash("hello"));

        // Different input should produce different output
        assert_ne!(hash("hello"), hash("world"));

        // Carriage returns should be stripped
        assert_eq!(hash("hello\r\nworld"), hash("hello\nworld"));
    }
}
