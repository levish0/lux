use std::borrow::Cow;
use super::entities_data::ENTITIES;

const WINDOWS_1252: [u32; 32] = [
    8364, 129, 8218, 402, 8222, 8230, 8224, 8225, 710, 8240, 352, 8249, 338, 141, 381, 143, 144,
    8216, 8217, 8220, 8221, 8226, 8211, 8212, 732, 8482, 353, 8250, 339, 157, 382, 376,
];

/// Validate a code point, applying Svelte-specific transformations.
fn validate_code(code: u32) -> u32 {
    // Line feed becomes space
    if code == 10 {
        return 32;
    }
    // ASCII range
    if code < 128 {
        return code;
    }
    // Windows-1252 legacy codes (128-159)
    if code <= 159 {
        return WINDOWS_1252[(code - 128) as usize];
    }
    // Basic multilingual plane (before surrogates)
    if code < 55296 {
        return code;
    }
    // UTF-16 surrogate halves
    if code <= 57343 {
        return 0;
    }
    // Rest of basic multilingual plane
    if code <= 65535 {
        return code;
    }
    // Supplementary multilingual plane 0x10000 - 0x1ffff
    if (65536..=131071).contains(&code) {
        return code;
    }
    // Supplementary ideographic plane 0x20000 - 0x2ffff
    if (131072..=196607).contains(&code) {
        return code;
    }
    // Supplementary special-purpose plane
    if (917504..=917631).contains(&code) || (917760..=917999).contains(&code) {
        return code;
    }
    0
}

/// Decode HTML character references in a string.
/// `is_attribute_value` controls whether entities without semicolons are decoded
/// when followed by alphanumeric, underscore, or equals characters.
/// 
/// Returns `Cow::Borrowed` if no entities are found (zero-copy),
/// or `Cow::Owned` if decoding was performed.
pub fn decode_character_references(html: &'_ str, is_attribute_value: bool) -> Cow<'_, str> {
    // Fast path: no ampersand means no entities
    if !html.contains('&') {
        return Cow::Borrowed(html);
    }

    let bytes = html.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;
    let mut modified = false;

    while i < len {
        if bytes[i] != b'&' {
            // Fast path: copy non-entity characters
            let start = i;
            while i < len && bytes[i] != b'&' {
                i += 1;
            }
            result.push_str(&html[start..i]);
            continue;
        }

        // Found '&'
        let amp_pos = i;
        i += 1; // skip '&'

        if i >= len {
            result.push('&');
            break;
        }

        if bytes[i] == b'#' {
            // Numeric entity
            i += 1;
            if i < len && (bytes[i] == b'x' || bytes[i] == b'X') {
                // Hex: &#xHH;
                i += 1;
                let start = i;
                while i < len && bytes[i].is_ascii_hexdigit() {
                    i += 1;
                }
                if start == i {
                    result.push_str(&html[amp_pos..i]);
                    continue;
                }
                let code = u32::from_str_radix(&html[start..i], 16).unwrap_or(0);
                if i < len && bytes[i] == b';' {
                    i += 1;
                }
                if code == 0 {
                    result.push_str(&html[amp_pos..i]);
                } else {
                    let validated = validate_code(code);
                    if let Some(c) = char::from_u32(validated) {
                        result.push(c);
                        modified = true;
                    } else {
                        result.push_str(&html[amp_pos..i]);
                    }
                }
            } else {
                // Decimal: &#DD;
                let start = i;
                while i < len && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                if start == i {
                    result.push_str(&html[amp_pos..i]);
                    continue;
                }
                let code = html[start..i].parse::<u32>().unwrap_or(0);
                if i < len && bytes[i] == b';' {
                    i += 1;
                }
                if code == 0 {
                    result.push_str(&html[amp_pos..i]);
                } else {
                    let validated = validate_code(code);
                    if let Some(c) = char::from_u32(validated) {
                        result.push(c);
                        modified = true;
                    } else {
                        result.push_str(&html[amp_pos..i]);
                    }
                }
            }
        } else if bytes[i].is_ascii_alphanumeric() {
            // Named entity
            let name_start = i;
            while i < len && bytes[i].is_ascii_alphanumeric() {
                i += 1;
            }
            let has_semicolon = i < len && bytes[i] == b';';
            let candidate_end = if has_semicolon { i + 1 } else { i };
            let candidate = &html[name_start..candidate_end];

            // Try longest prefix match
            let mut matched_len: usize = 0;
            let mut matched_code: u32 = 0;

            for n in (1..=candidate.len()).rev() {
                let prefix = &candidate[..n];
                if let Some(&code) = ENTITIES.get(prefix) {
                    // For attribute values: entities without ';' need word boundary + not '='
                    if is_attribute_value && !prefix.ends_with(';') {
                        let after_pos = name_start + n;
                        if after_pos < len {
                            let next_byte = bytes[after_pos];
                            if next_byte.is_ascii_alphanumeric()
                                || next_byte == b'_'
                                || next_byte == b'='
                            {
                                continue;
                            }
                        }
                    }
                    matched_len = n;
                    matched_code = code;
                    break;
                }
            }

            if matched_len > 0 {
                let validated = validate_code(matched_code);
                if let Some(c) = char::from_u32(validated) {
                    result.push(c);
                    modified = true;
                } else {
                    result.push_str(&html[amp_pos..amp_pos + 1 + matched_len]);
                }
                i = name_start + matched_len;
            } else {
                // No entity matched, output '&' and re-scan from name_start
                result.push('&');
                i = name_start;
            }
        } else {
            // '&' followed by non-alphanumeric, non-'#'
            result.push('&');
        }
    }

    if modified {
        Cow::Owned(result)
    } else {
        // No actual decoding happened, return borrowed original
        Cow::Borrowed(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_named_entities() {
        assert_eq!(decode_character_references("&amp;", false), "&");
        assert_eq!(decode_character_references("&lt;", false), "<");
        assert_eq!(decode_character_references("&gt;", false), ">");
        assert_eq!(decode_character_references("&nbsp;", false), "\u{00a0}");
        assert_eq!(decode_character_references("&quot;", false), "\"");
    }

    #[test]
    fn test_numeric_entities() {
        assert_eq!(decode_character_references("&#65;", false), "A");
        assert_eq!(decode_character_references("&#x41;", false), "A");
        assert_eq!(decode_character_references("&#x41", false), "A");
        assert_eq!(decode_character_references("&#97;", false), "a");
    }

    #[test]
    fn test_validate_code_lf() {
        assert_eq!(decode_character_references("&#10;", false), " ");
    }

    #[test]
    fn test_validate_code_windows_1252() {
        assert_eq!(decode_character_references("&#128;", false), "\u{20ac}");
    }

    #[test]
    fn test_no_semicolon_entities() {
        assert_eq!(decode_character_references("&amp", false), "&");
        assert_eq!(decode_character_references("&lt", false), "<");
        assert_eq!(decode_character_references("&not", false), "\u{00ac}");
    }

    #[test]
    fn test_attribute_value_no_semicolon() {
        assert_eq!(decode_character_references("&notit", true), "&notit");
        assert_eq!(decode_character_references("&not ", true), "\u{00ac} ");
        assert_eq!(decode_character_references("&not=", true), "&not=");
    }

    #[test]
    fn test_mixed_content() {
        assert_eq!(
            decode_character_references("Hello &amp; world &lt;3", false),
            "Hello & world <3"
        );
    }

    #[test]
    fn test_no_entities() {
        assert_eq!(
            decode_character_references("Hello world", false),
            "Hello world"
        );
    }

    #[test]
    fn test_invalid_entity() {
        assert_eq!(decode_character_references("&foobar;", false), "&foobar;");
    }
}
