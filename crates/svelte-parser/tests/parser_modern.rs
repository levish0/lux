use std::fs;
use std::path::Path;

use serde_json::Value;
use svelte_parser::{ParseOptions, parse};

fn fixture_dir() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/parser-modern/samples"
    ))
}

fn parse_fixture(name: &str) {
    let dir = fixture_dir().join(name);
    let input_path = dir.join("input.svelte");
    let source = fs::read_to_string(&input_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", input_path.display(), e))
        .replace("\r\n", "\n")
        .trim_end()
        .to_string();

    let options = if name.starts_with("loose") {
        ParseOptions { loose: true }
    } else {
        ParseOptions::default()
    };

    let root = parse(&source, options)
        .unwrap_or_else(|errs| panic!("Parse failed for {}: {:?}", name, errs));

    // Serialize and strip "comments" like Svelte's test.ts
    let mut actual: Value = serde_json::to_value(&root).unwrap();
    if let Value::Object(ref mut obj) = actual {
        obj.remove("comments");
    }

    // Save _actual.json for debugging
    let actual_json = serde_json::to_string_pretty(&actual).unwrap();
    fs::write(dir.join("_actual.json"), &actual_json).unwrap();

    // Compare against reference output.json
    let expected_path = dir.join("output.json");
    let expected_str = fs::read_to_string(&expected_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", expected_path.display(), e));
    let mut expected: Value = serde_json::from_str(&expected_str).unwrap();

    // Strip "loc" fields from both (not implemented yet)
    strip_field(&mut actual, "loc");
    strip_field(&mut expected, "loc");

    assert_eq!(actual, expected, "Mismatch for fixture '{}'", name);
}

fn strip_field(value: &mut Value, field: &str) {
    match value {
        Value::Object(obj) => {
            obj.remove(field);
            for (_, v) in obj.iter_mut() {
                strip_field(v, field);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                strip_field(v, field);
            }
        }
        _ => {}
    }
}

#[test]
fn fixture_if_block() {
    parse_fixture("if-block");
}

#[test]
fn fixture_if_block_else() {
    parse_fixture("if-block-else");
}

#[test]
fn fixture_if_block_elseif() {
    parse_fixture("if-block-elseif");
}

#[test]
fn fixture_template_shadowroot() {
    parse_fixture("template-shadowroot");
}

#[test]
fn fixture_attachments() {
    parse_fixture("attachments");
}

#[test]
fn fixture_comment_before_function_binding() {
    parse_fixture("comment-before-function-binding");
}

#[test]
fn fixture_comment_before_script() {
    parse_fixture("comment-before-script");
}

#[test]
fn fixture_css_nth_syntax() {
    parse_fixture("css-nth-syntax");
}

#[test]
fn fixture_css_pseudo_classes() {
    parse_fixture("css-pseudo-classes");
}

#[test]
fn fixture_each_block_object_pattern() {
    parse_fixture("each-block-object-pattern");
}

#[test]
fn fixture_each_block_object_pattern_special_characters() {
    parse_fixture("each-block-object-pattern-special-characters");
}

#[test]
fn fixture_generic_snippets() {
    parse_fixture("generic-snippets");
}

#[test]
fn fixture_loose_invalid_block() {
    parse_fixture("loose-invalid-block");
}

#[test]
fn fixture_loose_invalid_expression() {
    parse_fixture("loose-invalid-expression");
}

#[test]
fn fixture_loose_unclosed_open_tag() {
    parse_fixture("loose-unclosed-open-tag");
}

#[test]
fn fixture_loose_unclosed_tag() {
    parse_fixture("loose-unclosed-tag");
}

#[test]
fn fixture_loose_valid_each_as() {
    parse_fixture("loose-valid-each-as");
}

#[test]
fn fixture_options() {
    parse_fixture("options");
}

#[test]
fn fixture_script_style_no_markup() {
    parse_fixture("script-style-no-markup");
}

#[test]
fn fixture_semicolon_inside_quotes() {
    parse_fixture("semicolon-inside-quotes");
}

#[test]
fn fixture_snippets() {
    parse_fixture("snippets");
}

#[test]
fn fixture_typescript_in_event_handler() {
    parse_fixture("typescript-in-event-handler");
}
