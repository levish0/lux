use super::CssParser;

impl<'a> CssParser<'a> {
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
