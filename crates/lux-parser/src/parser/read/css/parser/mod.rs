use lux_ast::common::Span;
use winnow::Result;
use winnow::error::ContextError;

mod identifier;
mod value;
mod whitespace;

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
}
