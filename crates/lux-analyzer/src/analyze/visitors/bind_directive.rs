//! BindDirective visitor for analysis.

use lux_ast::attributes::BindDirective;
use lux_utils::{fuzzymatch, get_binding_property, is_content_editable_binding, is_svg};

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;

/// Validates a bind directive.
pub fn visit_bind_directive(node: &BindDirective<'_>, state: &mut AnalysisState<'_, '_>, path: &[NodeKind<'_>]) {
    let binding_name = node.name;

    // Get immediate parent (like context.path.at(-1) in Svelte)
    let parent = path.last();

    // Check for bind:this on components
    if binding_name == "this" {
        if let Some(p) = parent {
            if matches!(
                p,
                NodeKind::Component(_) | NodeKind::SvelteComponent | NodeKind::SvelteSelf
            ) {
                state.analysis.uses_component_bindings = true;
            }
        }
        return;
    }

    // Validate binding against element type
    // Only validate on elements that can have bindings (matches reference BindDirective.js)
    if let Some(parent_node) = parent {
        let element_name = match parent_node {
            NodeKind::RegularElement(name) => Some(*name),
            NodeKind::SvelteElement => Some("svelte:element"),
            NodeKind::SvelteWindow => Some("svelte:window"),
            NodeKind::SvelteDocument => Some("svelte:document"),
            NodeKind::SvelteBody => Some("svelte:body"),
            _ => None,
        };

        if let Some(name) = element_name {
            validate_element_binding(node, state, name);
        }
    }
}

/// Validates a binding against an element.
fn validate_element_binding(node: &BindDirective<'_>, state: &mut AnalysisState<'_, '_>, element: &str) {
    let binding_name = node.name;

    if let Some(property) = get_binding_property(binding_name) {
        // Check valid_elements
        if let Some(valid) = property.valid_elements {
            if !valid.contains(&element) {
                let valid_list = valid
                    .iter()
                    .map(|e| format!("`<{}>`", e))
                    .collect::<Vec<_>>()
                    .join(", ");
                state.analysis.error(errors::bind_invalid_target(
                    node.span.into(),
                    binding_name,
                    &valid_list,
                ));
            }
        }

        // Check invalid_elements
        if let Some(invalid) = property.invalid_elements {
            if invalid.contains(&element) {
                state.analysis.error(errors::bind_invalid_name(
                    node.span.into(),
                    binding_name,
                    None,
                ));
            }
        }

        // Special case: offsetWidth on SVG
        if binding_name == "offsetWidth" && is_svg(element) {
            state.analysis.error(errors::bind_invalid_target(
                node.span.into(),
                binding_name,
                "non-`<svg>` elements. Use `bind:clientWidth` for `<svg>` instead",
            ));
        }

        // Content editable bindings require contenteditable attribute
        if is_content_editable_binding(binding_name) {
            // Note: Full validation requires checking for contenteditable attribute
            // which would need access to the element's attributes
        }
    } else {
        // Unknown binding
        let suggestion = find_similar_binding(binding_name);
        state.analysis.error(errors::bind_invalid_name(
            node.span.into(),
            binding_name,
            suggestion.as_deref(),
        ));
    }
}

/// Finds a similar known binding name for error suggestions.
fn find_similar_binding(name: &str) -> Option<String> {
    const KNOWN_BINDINGS: &[&str] = &[
        "value",
        "checked",
        "group",
        "files",
        "this",
        "clientWidth",
        "clientHeight",
        "offsetWidth",
        "offsetHeight",
        "innerWidth",
        "innerHeight",
        "scrollX",
        "scrollY",
        "textContent",
        "innerHTML",
        "innerText",
        "currentTime",
        "duration",
        "paused",
        "volume",
        "muted",
        "playbackRate",
        "seeking",
        "ended",
        "buffered",
        "seekable",
        "played",
        "readyState",
        "videoWidth",
        "videoHeight",
        "naturalWidth",
        "naturalHeight",
        "focused",
        "open",
        "indeterminate",
    ];

    fuzzymatch(name, KNOWN_BINDINGS).map(|s| format!("Did you mean '{}'?", s))
}
