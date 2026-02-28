use lux_ast::template::root::Root;

use super::template::render_fragment_template;

pub(super) fn render(
    root: &Root<'_>,
    css: Option<&str>,
    css_hash: Option<&str>,
    css_scope: Option<&str>,
) -> String {
    let template_result = render_fragment_template(&root.fragment);

    let template_literal = js_string_literal(&template_result.html);
    let css_literal = js_optional_string_literal(css);
    let css_hash_literal = js_optional_string_literal(css_hash);
    let css_scope_literal = js_optional_string_literal(css_scope);
    let has_dynamic_literal = if template_result.has_dynamic {
        "true"
    } else {
        "false"
    };

    format!(
        "const __lux_template = {template_literal};\n\
const __lux_css = {css_literal};\n\
const __lux_css_hash = {css_hash_literal};\n\
const __lux_css_scope = {css_scope_literal};\n\
const __lux_has_dynamic = {has_dynamic_literal};\n\
\n\
export {{ __lux_template as template, __lux_css as css, __lux_css_hash as cssHash, __lux_css_scope as cssScope, __lux_has_dynamic as hasDynamic }};\n\
\n\
export default {{\n\
  template: __lux_template,\n\
  css: __lux_css,\n\
  cssHash: __lux_css_hash,\n\
  cssScope: __lux_css_scope,\n\
  hasDynamic: __lux_has_dynamic,\n\
  render(_props = {{}}) {{\n\
    return __lux_template;\n\
  }}\n\
}};\n"
    )
}

fn js_optional_string_literal(value: Option<&str>) -> String {
    match value {
        Some(value) => js_string_literal(value),
        None => "null".to_string(),
    }
}

fn js_string_literal(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');

    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            _ => out.push(ch),
        }
    }

    out.push('"');
    out
}
