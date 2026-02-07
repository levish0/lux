/// CSS scoping hash generation.
///
/// Algorithm: DJB2 variant matching Svelte's `hash()` in `utils.js`.
/// Iterates UTF-16 code units in reverse, XORing with the running hash.
/// Output is an unsigned 32-bit integer formatted as base-36.

/// Generate a hash string for CSS scoping.
///
/// Matches Svelte's behavior exactly:
/// 1. Remove `\r` characters
/// 2. Convert to UTF-16 code units (matching JS `charCodeAt`)
/// 3. Iterate in reverse with DJB2
/// 4. Output as base-36 unsigned integer
pub fn hash(input: &str) -> String {
    let mut h: u32 = 5381;
    let units: Vec<u16> = input.encode_utf16().filter(|&u| u != 0x000D).collect();
    for &unit in units.iter().rev() {
        h = (h << 5).wrapping_sub(h) ^ (unit as u32);
    }
    base36(h)
}

fn base36(mut n: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    const CHARS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut result = Vec::new();
    while n > 0 {
        result.push(CHARS[(n % 36) as usize]);
        n /= 36;
    }
    result.reverse();
    String::from_utf8(result).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_basic() {
        // Verify deterministic output
        let h1 = hash("div { color: red; }");
        let h2 = hash("div { color: red; }");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_different_input() {
        let h1 = hash("div { color: red; }");
        let h2 = hash("div { color: blue; }");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_carriage_return_stripped() {
        let h1 = hash("div { color: red; }");
        let h2 = hash("div { color: red; }\r");
        assert_eq!(h1, h2);

        let h3 = hash("div {\r\n  color: red;\r\n}");
        let h4 = hash("div {\n  color: red;\n}");
        assert_eq!(h3, h4);
    }

    #[test]
    fn test_hash_empty() {
        let h = hash("");
        assert_eq!(h, base36(5381));
    }

    #[test]
    fn test_base36() {
        assert_eq!(base36(0), "0");
        assert_eq!(base36(35), "z");
        assert_eq!(base36(36), "10");
        assert_eq!(base36(5381), "45h");
    }
}
