//! Second pass: Semantic analysis.
//!
//! This module performs semantic analysis on the AST after scope creation:
//! - Validates rune usage and placement
//! - Detects runes mode vs legacy mode
//! - Collects component metadata
//! - Analyzes CSS
//! - Orders reactive statements

pub mod a11y;
mod analysis;
mod css;
pub mod errors;
mod state;
pub mod utils;
mod visitor;
pub mod visitors;
pub mod warnings;

pub use analysis::{Analysis, ComponentAnalysis, CssAnalysis, Export, ReactiveStatement};
pub use state::{AnalysisState, AstType};
use visitor::run_analysis;

use lux_ast::root::Root;
use lux_utils::{hash, is_rune};

use crate::scope::create_scopes;

/// Options for component analysis.
#[derive(Debug, Clone, Default)]
pub struct AnalyzeOptions {
    /// The filename of the component
    pub filename: String,
    /// Whether runes mode is explicitly enabled/disabled (None = auto-detect)
    pub runes: Option<bool>,
    /// Whether to generate accessors
    pub accessors: bool,
    /// Whether immutable mode is enabled
    pub immutable: bool,
    /// CSS hash function configuration
    pub css_hash: Option<String>,
    /// Whether this is a custom element
    pub custom_element: bool,
}

/// Analyzes a Svelte component.
///
/// This is the main entry point for semantic analysis. It:
/// 1. Creates the scope tree (first pass)
/// 2. Walks the AST to collect metadata and validate (second pass)
/// 3. Returns the complete analysis result
pub fn analyze_component<'s, 'a>(source: &'s str, root: &mut Root<'a>, options: AnalyzeOptions) -> ComponentAnalysis<'s> {
    // First pass: create scopes
    let scope_result = create_scopes(root);
    let scope_tree = scope_result.scopes;

    // Determine component name from filename
    let name = get_component_name(&options.filename);

    // Create the analysis result
    let template_scope = scope_tree.root_scope_id();
    let mut analysis = ComponentAnalysis::new(source, name.clone(), scope_tree, template_scope);

    // Detect runes mode
    analysis.runes = detect_runes_mode(root, &analysis, options.runes, scope_result.has_await);

    // Set other options
    analysis.base.immutable = analysis.runes || options.immutable;
    analysis.base.accessors = options.custom_element || (!analysis.runes && options.accessors);
    analysis.custom_element = options.custom_element;
    analysis.inject_styles = options.custom_element; // TODO: also check css option

    // Analyze CSS if present
    if let Some(stylesheet) = &root.css {
        let css_content = stylesheet.content.styles;
        analysis.css = css::analyze_css(stylesheet);
        analysis.css.hash = compute_css_hash(&name, css_content, &options.filename);
    }

    // Second pass: walk AST and analyze
    run_analysis(root, &mut analysis);

    // Post-analysis checks
    check_mixed_event_syntax(&analysis);

    analysis
}

/// Checks for mixed event handler syntax (on:click vs onclick).
fn check_mixed_event_syntax(analysis: &ComponentAnalysis<'_>) {
    if analysis.event_directive_span.is_some() && analysis.uses_event_attributes {
        // TODO: Emit error for mixed event handler syntaxes
    }
}

/// Gets the component name from a filename.
fn get_component_name(filename: &str) -> String {
    let parts: Vec<&str> = filename.split(['/', '\\']).collect();
    let basename = parts.last().unwrap_or(&"Component");
    let last_dir = if parts.len() > 1 {
        parts.get(parts.len() - 2)
    } else {
        None
    };

    let mut name = basename.replace(".svelte", "");

    // If the file is named "index", use the directory name instead
    if name == "index" {
        if let Some(dir) = last_dir {
            if *dir != "src" {
                name = dir.to_string();
            }
        }
    }

    // Capitalize first letter
    let mut chars: Vec<char> = name.chars().collect();
    if let Some(first) = chars.first_mut() {
        *first = first.to_ascii_uppercase();
    }

    chars.into_iter().collect()
}

/// Detects whether runes mode should be enabled.
fn detect_runes_mode(
    _root: &Root<'_>,
    analysis: &ComponentAnalysis,
    explicit_runes: Option<bool>,
    has_await: bool,
) -> bool {
    // If explicitly set, use that
    if let Some(runes) = explicit_runes {
        return runes;
    }

    // Check for top-level await (implies runes mode)
    if has_await {
        return true;
    }

    // Check for rune usage in references across ALL scopes
    // Runes like $state, $derived are referenced in the instance script scope
    for (_, scope) in analysis.scope_tree.iter_scopes() {
        for name in scope.references.keys() {
            if is_rune(name) {
                return true;
            }
        }
    }

    false
}

/// Computes the CSS hash for scoping.
fn compute_css_hash(_component_name: &str, css: &str, filename: &str) -> String {
    // Default hash format: svelte-{hash}
    let input = format!("{}{}", filename, css);
    format!("svelte-{}", hash(&input))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_component_name() {
        assert_eq!(get_component_name("Button.svelte"), "Button");
        assert_eq!(get_component_name("src/Button.svelte"), "Button");
        assert_eq!(get_component_name("components/button/index.svelte"), "Button");
        assert_eq!(get_component_name("src/index.svelte"), "Index");
    }
}
