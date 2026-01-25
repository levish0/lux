//! Accessibility (a11y) validation for Svelte components.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/a11y/`
//!
//! This module provides accessibility checks for HTML elements in Svelte templates.
//! It validates ARIA attributes, role usage, interactive elements, and element-specific
//! accessibility requirements.

pub mod constants;

use lux_ast::attributes::{Attribute, AttributeValue};
use lux_ast::node::AttributeNode;
use oxc_ast::ast::Expression;
use oxc_span::Span;
use rustc_hash::{FxHashMap, FxHashSet};

use self::constants::*;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::warnings;

/// Elements that should not have ARIA attributes.
const INVISIBLE_ELEMENTS: &[&str] = &["meta", "html", "script", "style"];

/// Heading tags regex pattern.
fn is_heading_tag(name: &str) -> bool {
    matches!(name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
}

/// Get static text value from an attribute.
fn get_static_value(attr: &Attribute<'_>) -> Option<String> {
    match &attr.value {
        AttributeValue::True => Some(String::new()),
        AttributeValue::ExpressionTag(tag) => {
            if let Expression::StringLiteral(lit) = &tag.expression {
                Some(lit.value.to_string())
            } else {
                None
            }
        }
        AttributeValue::Sequence(seq) => {
            let mut result = String::new();
            for item in seq {
                match item {
                    lux_ast::attributes::AttributeSequenceValue::Text(t) => {
                        result.push_str(t.data);
                    }
                    lux_ast::attributes::AttributeSequenceValue::ExpressionTag(_) => {
                        return None; // Dynamic, can't get static value
                    }
                }
            }
            Some(result)
        }
    }
}

/// Check an element for a11y issues.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/shared/a11y/index.js`
pub fn check_element(
    element_name: &str,
    attributes: &[AttributeNode<'_>],
    state: &mut AnalysisState<'_, '_>,
    path: &[NodeKind<'_>],
    span: Span,
) {
    let mut attribute_map: FxHashMap<&str, &Attribute<'_>> = FxHashMap::default();
    let mut handlers: FxHashSet<&str> = FxHashSet::default();
    let mut has_spread = false;

    // Collect attributes and handlers
    for attr in attributes {
        match attr {
            AttributeNode::Attribute(a) => {
                if a.name.starts_with("on") && matches!(a.value, AttributeValue::ExpressionTag(_)) {
                    handlers.insert(&a.name[2..]);
                } else {
                    attribute_map.insert(a.name, a);
                }
            }
            AttributeNode::SpreadAttribute(_) => {
                has_spread = true;
            }
            AttributeNode::OnDirective(d) => {
                handlers.insert(d.name);
            }
            _ => {}
        }
    }

    // Check distracting elements
    if A11Y_DISTRACTING_ELEMENTS.contains(element_name) {
        state.analysis.warning(warnings::a11y_distracting_elements(span, element_name));
    }

    // Check for accesskey attribute
    if attribute_map.contains_key("accesskey") {
        if let Some(attr) = attribute_map.get("accesskey") {
            state.analysis.warning(warnings::a11y_accesskey(attr.span.into()));
        }
    }

    // Check for autofocus attribute
    if attribute_map.contains_key("autofocus") {
        if let Some(attr) = attribute_map.get("autofocus") {
            state.analysis.warning(warnings::a11y_autofocus(attr.span.into()));
        }
    }

    // Check scope attribute (only valid on th)
    if element_name != "th" && attribute_map.contains_key("scope") {
        if let Some(attr) = attribute_map.get("scope") {
            state.analysis.warning(warnings::a11y_misplaced_scope(attr.span.into()));
        }
    }

    // Check tabindex
    if let Some(attr) = attribute_map.get("tabindex") {
        if let Some(value) = get_static_value(attr) {
            if let Ok(tabindex) = value.parse::<i32>() {
                if tabindex > 0 {
                    state.analysis.warning(warnings::a11y_positive_tabindex(attr.span.into()));
                }
            }
        }
    }

    // Check ARIA attributes
    for attr in attributes {
        if let AttributeNode::Attribute(a) = attr {
            let name_lower = a.name.to_lowercase();

            if name_lower.starts_with("aria-") {
                // Check if element supports ARIA
                if INVISIBLE_ELEMENTS.contains(&element_name) {
                    state.analysis.warning(warnings::a11y_aria_attributes(a.span.into(), element_name));
                }

                // Check for unknown ARIA attribute
                let aria_name = &name_lower[5..];
                if !ARIA_ATTRIBUTES.contains(aria_name) {
                    // Try to find a suggestion
                    let suggestion = ARIA_ATTRIBUTES.iter()
                        .find(|&&known| levenshtein(aria_name, known) <= 2)
                        .copied();
                    state.analysis.warning(warnings::a11y_unknown_aria_attribute(
                        a.span.into(),
                        aria_name,
                        suggestion,
                    ));
                }

                // Check aria-hidden on heading
                if name_lower == "aria-hidden" && is_heading_tag(element_name) {
                    state.analysis.warning(warnings::a11y_hidden(a.span.into(), element_name));
                }

                // Check aria-activedescendant has tabindex
                if name_lower == "aria-activedescendant"
                    && !has_spread
                    && !attribute_map.contains_key("tabindex")
                {
                    state.analysis.warning(warnings::a11y_aria_activedescendant_has_tabindex(a.span.into()));
                }
            }
        }
    }

    // Check role attribute
    if let Some(attr) = attribute_map.get("role") {
        // Check if element supports role
        if INVISIBLE_ELEMENTS.contains(&element_name) {
            state.analysis.warning(warnings::a11y_misplaced_role(attr.span.into(), element_name));
        }

        if let Some(value) = get_static_value(attr) {
            for role in value.split_whitespace() {
                // Check for abstract role
                if ABSTRACT_ROLES.contains(role) {
                    state.analysis.warning(warnings::a11y_no_abstract_role(attr.span.into(), role));
                }
                // Check for unknown role
                else if !ARIA_ROLES.contains(role) {
                    let suggestion = ARIA_ROLES.iter()
                        .find(|&&known| levenshtein(role, known) <= 2)
                        .copied();
                    state.analysis.warning(warnings::a11y_unknown_role(attr.span.into(), role, suggestion));
                }

                // Check for redundant role
                if let Some(implicit) = A11Y_IMPLICIT_SEMANTICS.get(element_name) {
                    if *implicit == role
                        && !matches!(element_name, "ul" | "ol" | "li")
                        && !(element_name == "a" && !attribute_map.contains_key("href"))
                    {
                        state.analysis.warning(warnings::a11y_no_redundant_roles(attr.span.into(), role));
                    }
                }

                // Check nested redundant roles (header/footer in section/article)
                if !is_parent_section_or_article(path) {
                    if let Some(nested_role) = A11Y_NESTED_IMPLICIT_SEMANTICS.get(element_name) {
                        if *nested_role == role {
                            state.analysis.warning(warnings::a11y_no_redundant_roles(attr.span.into(), role));
                        }
                    }
                }
            }
        }
    }

    // Element-specific checks
    check_element_specific(element_name, &attribute_map, &handlers, has_spread, state, span);
}

/// Check element-specific a11y rules.
fn check_element_specific(
    element_name: &str,
    attribute_map: &FxHashMap<&str, &Attribute<'_>>,
    handlers: &FxHashSet<&str>,
    has_spread: bool,
    state: &mut AnalysisState<'_, '_>,
    span: Span,
) {
    match element_name {
        // img requires alt
        "img" => {
            if !has_spread && !attribute_map.contains_key("alt") {
                state.analysis.warning(warnings::a11y_missing_attribute(span, "img", "alt"));
            }
            // Check for redundant alt text
            if let Some(attr) = attribute_map.get("alt") {
                if let Some(value) = get_static_value(attr) {
                    let lower = value.to_lowercase();
                    if lower.contains("image") || lower.contains("picture") || lower.contains("photo") {
                        state.analysis.warning(warnings::a11y_img_redundant_alt(attr.span.into()));
                    }
                }
            }
        }

        // a requires href
        "a" => {
            if !has_spread && !attribute_map.contains_key("href") {
                state.analysis.warning(warnings::a11y_missing_attribute(span, "a", "href"));
            }
        }

        // area requires alt
        "area" => {
            if !has_spread
                && !attribute_map.contains_key("alt")
                && !attribute_map.contains_key("aria-label")
                && !attribute_map.contains_key("aria-labelledby")
            {
                state.analysis.warning(warnings::a11y_missing_attribute(span, "area", "alt"));
            }
        }

        // html requires lang
        "html" => {
            if !has_spread && !attribute_map.contains_key("lang") {
                state.analysis.warning(warnings::a11y_missing_attribute(span, "html", "lang"));
            }
        }

        // iframe requires title
        "iframe" => {
            if !has_spread && !attribute_map.contains_key("title") {
                state.analysis.warning(warnings::a11y_missing_attribute(span, "iframe", "title"));
            }
        }

        // object requires title or aria-label
        "object" => {
            if !has_spread
                && !attribute_map.contains_key("title")
                && !attribute_map.contains_key("aria-label")
                && !attribute_map.contains_key("aria-labelledby")
            {
                state.analysis.warning(warnings::a11y_missing_attribute(span, "object", "title"));
            }
        }

        // video/audio require captions
        "video" | "audio" => {
            // This check requires examining children for <track kind="captions">
            // We can't easily do this without more context, so skip for now
        }

        // headings require content
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            // This requires checking children, which we don't have direct access to
        }

        _ => {}
    }

    // Check for mouse events without keyboard events
    if handlers.contains("mouseover") && !handlers.contains("focus") {
        state.analysis.warning(warnings::a11y_mouse_events_have_key_events(
            span, "mouseover", "focus"
        ));
    }
    if handlers.contains("mouseout") && !handlers.contains("blur") {
        state.analysis.warning(warnings::a11y_mouse_events_have_key_events(
            span, "mouseout", "blur"
        ));
    }
}

/// Check if parent is section or article.
fn is_parent_section_or_article(path: &[NodeKind<'_>]) -> bool {
    for node in path.iter().rev() {
        if let NodeKind::RegularElement(name) = node {
            if *name == "section" || *name == "article" {
                return true;
            }
        }
    }
    false
}

/// Simple Levenshtein distance for fuzzy matching.
fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}
