use winnow::Result;
use winnow::error::ContextError;

use super::CssParser;

impl<'a> CssParser<'a> {
    pub fn read_identifier(&mut self) -> Result<&'a str> {
        let start = self.index;

        if self.index < self.source.len() {
            let b = self.source.as_bytes()[self.index];
            if b == b'-' && self.index + 1 < self.source.len() {
                let next = self.source.as_bytes()[self.index + 1];
                if next.is_ascii_digit() {
                    return Err(ContextError::new());
                }
            } else if b.is_ascii_digit() {
                return Err(ContextError::new());
            }
        }

        while self.index < self.source.len() {
            let b = self.source.as_bytes()[self.index];
            if b == b'\\' {
                // CSS escape sequence.
                self.index += 1;
                // \HHHHHH optional whitespace.
                let hex_start = self.index;
                while self.index < self.source.len()
                    && self.index - hex_start < 6
                    && self.source.as_bytes()[self.index].is_ascii_hexdigit()
                {
                    self.index += 1;
                }
                if self.index > hex_start {
                    // Optional trailing whitespace after hex escape.
                    if self.index < self.source.len()
                        && self.source.as_bytes()[self.index].is_ascii_whitespace()
                    {
                        self.index += 1;
                    }
                } else if self.index < self.source.len() {
                    // Escaped non-hex character.
                    self.index += 1;
                }
            } else if b >= 160 || b.is_ascii_alphanumeric() || b == b'_' || b == b'-' {
                self.index += 1;
            } else {
                // Check for multi-byte chars with codepoint >= 160.
                let ch = self.source[self.index..]
                    .chars()
                    .next()
                    .expect("index always within source bounds");
                if ch as u32 >= 160 {
                    self.index += ch.len_utf8();
                } else {
                    break;
                }
            }
        }

        if self.index == start {
            return Err(ContextError::new());
        }

        Ok(&self.source[start..self.index])
    }
}
