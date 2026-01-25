use oxc_ast::ast::{BindingPattern, Expression, FormalParameter, FormalParameterRest, Program};
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

/// Shift all spans in an Expression by `offset` bytes.
pub fn shift_expression_spans(expr: &mut Expression<'_>, offset: u32) {
    let mut shifter = SpanShifter { offset };
    shifter.visit_expression(expr);
}

/// Shift all spans in a BindingPattern by `offset` bytes.
#[allow(dead_code)]
pub fn shift_binding_pattern_spans(pattern: &mut BindingPattern<'_>, offset: u32) {
    let mut shifter = SpanShifter { offset };
    shifter.visit_binding_pattern(pattern);
}

/// Shift all spans in a FormalParameter by `offset` bytes.
pub fn shift_formal_parameter_spans(param: &mut FormalParameter<'_>, offset: u32) {
    let mut shifter = SpanShifter { offset };
    shifter.visit_formal_parameter(param);
}

/// Shift all spans in a FormalParameterRest by `offset` bytes.
pub fn shift_formal_parameter_rest_spans(rest: &mut FormalParameterRest<'_>, offset: u32) {
    let mut shifter = SpanShifter { offset };
    shifter.visit_formal_parameter_rest(rest);
}

/// Shift all spans in a Program by `offset` bytes.
pub fn shift_program_spans(program: &mut Program<'_>, offset: u32) {
    let mut shifter = SpanShifter { offset };
    shifter.visit_program(program);
}
