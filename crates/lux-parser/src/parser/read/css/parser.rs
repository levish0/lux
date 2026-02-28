use lux_ast::common::Span;
use winnow::Result;
use winnow::error::ContextError;

pub struct CssParser<'a> {
    pub source: &'a str,
    pub index: usize,
    pub offset: u32,
}

impl<'a> CssParser<'a> {
    pub fn new(source: &'a str, offset: u32) -> Self {
        Self {
            source,
            index: 0,
            offset,
        }
    }

    pub fn span(&self, start: usize, end: usize) -> Span {
        Span::new(start as u32 + self.offset, end as u32 + self.offset)
    }

    pub fn remaining(&self) -> &'a str {
        &self.source[self.index..]
    }

    pub fn at_end(&self) -> bool {
        self.index >= self.source.len()
    }

    pub fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.index).copied()
    }

    pub fn matches(&self, s: &str) -> bool {
        self.remaining().starts_with(s)
    }

    pub fn eat(&mut self, s: &str) -> bool {
        if self.remaining().starts_with(s) {
            self.index += s.len();
            true
        } else {
            false
        }
    }

    pub fn eat_required(&mut self, s: &str) -> Result<()> {
        if self.eat(s) {
            Ok(())
        } else {
            Err(ContextError::new())
        }
    }

    pub fn skip_whitespace(&mut self) {
        while self.index < self.source.len()
            && self.source.as_bytes()[self.index].is_ascii_whitespace()
        {
            self.index += 1;
        }
    }

    pub fn skip_ws_and_comments(&mut self) {
        self.skip_whitespace();
        while self.matches("/*") || self.matches("<!--") {
            if self.eat("/*")
                && let Some(end) = self.source[self.index..].find("*/")
            {
                self.index += end + 2;
            }
            if self.eat("<!--")
                && let Some(end) = self.source[self.index..].find("-->")
            {
                self.index += end + 3;
            }
            self.skip_whitespace();
        }
    }

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
                // CSS escape sequence
                self.index += 1;
                // \HHHHHH optional whitespace
                let hex_start = self.index;
                while self.index < self.source.len()
                    && self.index - hex_start < 6
                    && self.source.as_bytes()[self.index].is_ascii_hexdigit()
                {
                    self.index += 1;
                }
                if self.index > hex_start {
                    // Optional trailing whitespace after hex escape
                    if self.index < self.source.len()
                        && self.source.as_bytes()[self.index].is_ascii_whitespace()
                    {
                        self.index += 1;
                    }
                } else if self.index < self.source.len() {
                    // Escaped non-hex character
                    self.index += 1;
                }
            } else if b >= 160 || b.is_ascii_alphanumeric() || b == b'_' || b == b'-' {
                self.index += 1;
            } else {
                // Check for multi-byte chars with codepoint >= 160
                let ch = self.source[self.index..].chars().next().unwrap();
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

    pub fn read_css_value(&mut self) -> &'a str {
        let start = self.index;
        let bytes = self.source.as_bytes();
        let mut escaped = false;
        let mut in_url = false;
        let mut quote: Option<u8> = None;

        while self.index < bytes.len() {
            let b = bytes[self.index];

            if escaped {
                escaped = false;
                self.index += 1;
                continue;
            }

            if b == b'\\' {
                escaped = true;
                self.index += 1;
                continue;
            }

            if Some(b) == quote {
                quote = None;
            } else if b == b')' {
                in_url = false;
            } else if quote.is_none() && (b == b'"' || b == b'\'') {
                quote = Some(b);
            } else if b == b'('
                && self.index >= 3
                && &self.source[self.index - 3..self.index] == "url"
            {
                in_url = true;
            } else if (b == b';' || b == b'{' || b == b'}') && !in_url && quote.is_none() {
                return self.source[start..self.index].trim();
            }

            self.index += 1;
        }

        self.source[start..self.index].trim()
    }

    pub fn read_attribute_value(&mut self) -> &'a str {
        let mut escaped = false;
        let quote = if self.eat("\"") {
            Some(b'"')
        } else if self.eat("'") {
            Some(b'\'')
        } else {
            None
        };

        let start = self.index;
        let bytes = self.source.as_bytes();

        while self.index < bytes.len() {
            let b = bytes[self.index];
            if escaped {
                escaped = false;
                self.index += 1;
                continue;
            }
            if b == b'\\' {
                escaped = true;
                self.index += 1;
                continue;
            }
            match quote {
                Some(q) if b == q => {
                    let value = self.source[start..self.index].trim();
                    self.index += 1; // consume closing quote
                    return value;
                }
                None if b.is_ascii_whitespace() || b == b']' => {
                    return self.source[start..self.index].trim();
                }
                _ => {}
            }
            self.index += 1;
        }

        self.source[start..self.index].trim()
    }
}
