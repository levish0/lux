use super::entities_data::ENTITIES;
use std::borrow::Cow;

const WINDOWS_1252: [u32; 32] = [
    8364, 129, 8218, 402, 8222, 8230, 8224, 8225, 710, 8240, 352, 8249, 338, 141, 381, 143, 144,
    8216, 8217, 8220, 8221, 8226, 8211, 8212, 732, 8482, 353, 8250, 339, 157, 382, 376,
];

fn validate_code(code: u32) -> u32 {
    if code == 10 {
        return 32;
    }
    if code < 128 {
        return code;
    }
    if code <= 159 {
        return WINDOWS_1252[(code - 128) as usize];
    }
    if code < 55296 {
        return code;
    }
    if code <= 57343 {
        return 0;
    }
    if code <= 65535 {
        return code;
    }
    if (65536..=131071).contains(&code) {
        return code;
    }
    if (131072..=196607).contains(&code) {
        return code;
    }
    if (917504..=917631).contains(&code) || (917760..=917999).contains(&code) {
        return code;
    }
    0
}

pub fn decode_character_references(html: &str, is_attribute_value: bool) -> Cow<'_, str> {
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
            let start = i;
            while i < len && bytes[i] != b'&' {
                i += 1;
            }
            result.push_str(&html[start..i]);
            continue;
        }

        let amp_pos = i;
        i += 1;

        if i >= len {
            result.push('&');
            break;
        }

        if bytes[i] == b'#' {
            i += 1;
            if i < len && (bytes[i] == b'x' || bytes[i] == b'X') {
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
            let name_start = i;
            while i < len && bytes[i].is_ascii_alphanumeric() {
                i += 1;
            }
            let has_semicolon = i < len && bytes[i] == b';';
            let candidate_end = if has_semicolon { i + 1 } else { i };
            let candidate = &html[name_start..candidate_end];

            let mut matched_len: usize = 0;
            let mut matched_code: u32 = 0;

            for n in (1..=candidate.len()).rev() {
                let prefix = &candidate[..n];
                if let Some(&code) = ENTITIES.get(prefix) {
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
                result.push('&');
                i = name_start;
            }
        } else {
            result.push('&');
        }
    }

    if modified {
        Cow::Owned(result)
    } else {
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
