use lux_ast::common::Span;
use oxc_ast::ast::BindingPattern;
use oxc_span::GetSpan;
use winnow::Result;
use winnow::combinator::opt;
use winnow::prelude::*;
use winnow::stream::Location as _;
use winnow::token::literal;

use crate::input::Input;
use crate::parser::read::pattern::read_binding_pattern_until;
use crate::parser::utils::helpers::skip_whitespace;

pub(super) struct ParsedSnippetParams<'a> {
    pub parameters: Vec<BindingPattern<'a>>,
    pub rest_parameter_spans: Vec<Span>,
}

pub(super) fn parse_snippet_params<'a>(input: &mut Input<'a>) -> Result<ParsedSnippetParams<'a>> {
    let mut parameters = Vec::new();
    let mut rest_parameter_spans = Vec::new();

    loop {
        skip_whitespace(input);

        let remaining: &str = &input.input;
        if remaining.starts_with(')') {
            break;
        }

        let param_start = input.current_token_start() as u32;
        let has_rest_prefix = input.input.trim_start().starts_with("...");

        let pattern = read_binding_pattern_until(input, b",)")?;
        if has_rest_prefix {
            rest_parameter_spans.push(Span::new(param_start, pattern.span().end));
        }
        parameters.push(pattern);

        skip_whitespace(input);

        // Comma between params.
        if opt(literal(",")).parse_next(input)?.is_none() {
            break;
        }
    }

    Ok(ParsedSnippetParams {
        parameters,
        rest_parameter_spans,
    })
}
