use super::CssParser;

impl CssParser<'_> {
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
}
