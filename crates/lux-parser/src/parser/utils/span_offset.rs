use oxc_ast::ast::{BindingPattern, Expression};
use oxc_ast_visit::VisitMut;
use oxc_span::Span;

struct SpanShifter {
    offset: u32,
}

impl<'a> VisitMut<'a> for SpanShifter {
    fn visit_span(&mut self, span: &mut Span) {
        span.start += self.offset;
        span.end += self.offset;
    }
}

pub fn shift_expression_spans(expr: &mut Expression<'_>, offset: u32) {
    let mut shifter = SpanShifter { offset };
    shifter.visit_expression(expr);
}

pub fn shift_binding_pattern_spans(pattern: &mut BindingPattern<'_>, offset: u32) {
    let mut shifter = SpanShifter { offset };
    shifter.visit_binding_pattern(pattern);
}
