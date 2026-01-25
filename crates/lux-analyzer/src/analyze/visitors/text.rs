//! Text node visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/Text.js`

use lux_ast::text::Text;

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::warnings;

/// Text visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/Text.js`
pub fn visit_text(node: &Text<'_>, state: &mut AnalysisState<'_, '_>, path: &[NodeKind<'_>]) {
    // Check for valid text placement in HTML structure
    if let Some(parent_element) = get_parent_element_name(path) {
        // Only check non-whitespace text
        if !node.data.trim().is_empty() {
            if let Some(message) = is_tag_valid_with_parent("#text", parent_element) {
                state.analysis.error(errors::node_invalid_placement(
                    node.span.into(),
                    &message,
                ));
            }
        }
    }

    // Check for bidirectional control characters
    check_bidirectional_control_characters(node, state);
}

/// Gets the parent element name from the path if the immediate parent is a Fragment
/// and there's a parent element in the path.
fn get_parent_element_name<'a>(path: &'a [NodeKind<'a>]) -> Option<&'a str> {
    // The path should have Fragment as immediate parent
    if path.is_empty() {
        return None;
    }

    // Check if immediate parent is Fragment
    let parent = path.last()?;
    if !matches!(parent, NodeKind::Fragment) {
        return None;
    }

    // Find the element that owns this Fragment
    for node in path.iter().rev().skip(1) {
        match node {
            NodeKind::RegularElement(name) => return Some(*name),
            NodeKind::TitleElement => return Some("title"),
            NodeKind::SlotElement => return Some("slot"),
            NodeKind::SvelteHead => return Some("svelte:head"),
            NodeKind::SvelteBody => return Some("svelte:body"),
            NodeKind::SvelteWindow => return Some("svelte:window"),
            NodeKind::SvelteDocument => return Some("svelte:document"),
            _ => {}
        }
    }

    None
}

/// Checks if a tag is valid as a child of a parent element.
/// Returns Some(error_message) if invalid, None if valid.
fn is_tag_valid_with_parent(tag: &str, parent: &str) -> Option<String> {
    // Based on html-tree-validation.js
    // These are elements that will cause issues with SSR due to browser HTML repair

    match parent {
        // List items
        "li" => {
            if tag == "li" {
                return Some(format!(
                    "`<{}>` cannot be a direct child of `<{}>`, as it will close the parent element",
                    tag, parent
                ));
            }
            None
        }

        // Definition lists
        "dt" | "dd" => {
            if tag == "dt" || tag == "dd" {
                return Some(format!(
                    "`<{}>` cannot be a descendant of `<{}>`, as it will close the parent element",
                    tag, parent
                ));
            }
            None
        }

        // Paragraphs - many block elements will close them
        "p" => {
            let block_elements = [
                "address",
                "article",
                "aside",
                "blockquote",
                "div",
                "dl",
                "fieldset",
                "footer",
                "form",
                "h1",
                "h2",
                "h3",
                "h4",
                "h5",
                "h6",
                "header",
                "hgroup",
                "hr",
                "main",
                "menu",
                "nav",
                "ol",
                "p",
                "pre",
                "section",
                "table",
                "ul",
            ];
            if block_elements.contains(&tag) {
                return Some(format!(
                    "`<{}>` cannot be a descendant of `<{}>`, as it will close the parent element",
                    tag, parent
                ));
            }
            None
        }

        // Ruby annotations
        "rt" | "rp" => {
            if tag == "rt" || tag == "rp" {
                return Some(format!(
                    "`<{}>` cannot be a descendant of `<{}>`, as it will close the parent element",
                    tag, parent
                ));
            }
            None
        }

        // Select options
        "optgroup" => {
            if tag == "optgroup" {
                return Some(format!(
                    "`<{}>` cannot be a descendant of `<{}>`, as it will close the parent element",
                    tag, parent
                ));
            }
            None
        }
        "option" => {
            if tag == "option" || tag == "optgroup" {
                return Some(format!(
                    "`<{}>` cannot be a descendant of `<{}>`, as it will close the parent element",
                    tag, parent
                ));
            }
            None
        }

        // Table sections
        "thead" => {
            if tag == "tbody" || tag == "tfoot" {
                return Some(format!(
                    "`<{}>` cannot be a direct child of `<{}>`, as it will close the parent element",
                    tag, parent
                ));
            }
            None
        }
        "tbody" => {
            if tag == "tbody" || tag == "tfoot" {
                return Some(format!(
                    "`<{}>` cannot be a direct child of `<{}>`, as it will close the parent element",
                    tag, parent
                ));
            }
            None
        }
        "tfoot" => {
            if tag == "tbody" {
                return Some(format!(
                    "`<{}>` cannot be a direct child of `<{}>`, as it will close the parent element",
                    tag, parent
                ));
            }
            None
        }
        "tr" => {
            if tag == "tr" || tag == "tbody" {
                return Some(format!(
                    "`<{}>` cannot be a direct child of `<{}>`, as it will close the parent element",
                    tag, parent
                ));
            }
            None
        }
        "td" | "th" => {
            if tag == "td" || tag == "th" || tag == "tr" {
                return Some(format!(
                    "`<{}>` cannot be a direct child of `<{}>`, as it will close the parent element",
                    tag, parent
                ));
            }
            None
        }

        // Nested forms, anchors, buttons
        "form" => {
            if tag == "form" {
                return Some("`<form>` cannot be nested inside another `<form>`".to_string());
            }
            None
        }
        "a" => {
            if tag == "a" {
                return Some("`<a>` cannot be nested inside another `<a>`".to_string());
            }
            None
        }
        "button" => {
            if tag == "button" {
                return Some("`<button>` cannot be nested inside another `<button>`".to_string());
            }
            None
        }

        // Headings
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            if tag.starts_with('h') && tag.len() == 2 {
                return Some(format!(
                    "`<{}>` cannot be a descendant of `<{}>`, as it will close the parent element",
                    tag, parent
                ));
            }
            None
        }

        // Elements with restricted content models
        "table" => {
            let allowed = [
                "caption", "colgroup", "thead", "tbody", "tfoot", "tr", "script", "template",
            ];
            if tag != "#text" && !allowed.contains(&tag) {
                return Some(format!(
                    "`<{}>` cannot appear directly inside `<table>`. Wrap it in `<tbody>`, `<thead>`, or `<tfoot>`",
                    tag
                ));
            }
            None
        }

        "select" => {
            let allowed = ["option", "optgroup", "script", "template"];
            if tag != "#text" && !allowed.contains(&tag) {
                return Some(format!(
                    "`<{}>` cannot appear directly inside `<select>`",
                    tag
                ));
            }
            // Text is not allowed in select either
            if tag == "#text" {
                return Some(
                    "Text content is not allowed directly inside `<select>`".to_string(),
                );
            }
            None
        }

        _ => None,
    }
}

/// Checks for bidirectional control characters in text.
/// These can be used to create security vulnerabilities in code.
fn check_bidirectional_control_characters(node: &Text<'_>, state: &mut AnalysisState<'_, '_>) {
    // Unicode bidirectional control characters
    // U+202A LEFT-TO-RIGHT EMBEDDING
    // U+202B RIGHT-TO-LEFT EMBEDDING
    // U+202C POP DIRECTIONAL FORMATTING
    // U+202D LEFT-TO-RIGHT OVERRIDE
    // U+202E RIGHT-TO-LEFT OVERRIDE
    // U+2066 LEFT-TO-RIGHT ISOLATE
    // U+2067 RIGHT-TO-LEFT ISOLATE
    // U+2068 FIRST STRONG ISOLATE
    // U+2069 POP DIRECTIONAL ISOLATE

    let bidi_chars: &[char] = &[
        '\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{202E}', '\u{2066}', '\u{2067}',
        '\u{2068}', '\u{2069}',
    ];

    for (idx, c) in node.data.char_indices() {
        if bidi_chars.contains(&c) {
            let start = node.span.start + idx;
            let end = start + c.len_utf8();
            let span = oxc_span::Span::new(start as u32, end as u32);
            state
                .analysis
                .warning(warnings::bidirectional_control_characters(span));
        }
    }
}
