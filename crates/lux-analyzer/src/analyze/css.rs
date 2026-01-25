//! CSS analysis module.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/css/`
//!
//! This module performs CSS analysis:
//! - Collects keyframe names for scoping
//! - Detects :global rules
//! - Validates CSS selectors

use lux_ast::css::{
    CssAtrule, CssRule, SimpleSelector, StyleSheet, StyleSheetChild,
};

use super::analysis::CssAnalysis;

/// Analyze a CSS stylesheet.
pub fn analyze_css(stylesheet: &StyleSheet<'_>) -> CssAnalysis {
    let mut analysis = CssAnalysis::default();

    for child in &stylesheet.children {
        analyze_stylesheet_child(child, &mut analysis, false);
    }

    analysis
}

/// Analyze a stylesheet child (rule or atrule).
fn analyze_stylesheet_child(child: &StyleSheetChild, analysis: &mut CssAnalysis, in_global_block: bool) {
    match child {
        StyleSheetChild::Rule(rule) => {
            analyze_rule(rule, analysis, in_global_block);
        }
        StyleSheetChild::Atrule(atrule) => {
            analyze_atrule(atrule, analysis, in_global_block);
        }
    }
}

/// Analyze a CSS rule.
fn analyze_rule(rule: &CssRule, analysis: &mut CssAnalysis, in_global_block: bool) {
    // Check if this rule has :global selectors
    let has_global = rule_has_global(&rule.prelude);

    if has_global {
        analysis.has_global = true;
    }

    // Check for :global block (`:global { ... }`)
    let is_global_block = rule.prelude.children.iter().any(|complex| {
        complex.children.iter().any(|relative| {
            relative.selectors.iter().any(|sel| {
                matches!(sel, SimpleSelector::PseudoClassSelector(pseudo)
                    if pseudo.name == "global" && pseudo.args.is_none())
            })
        })
    });

    // Analyze children
    for child in &rule.block.children {
        match child {
            lux_ast::css::CssBlockChild::Rule(nested_rule) => {
                analyze_rule(nested_rule, analysis, in_global_block || is_global_block);
            }
            lux_ast::css::CssBlockChild::Atrule(atrule) => {
                analyze_atrule(atrule, analysis, in_global_block || is_global_block);
            }
            _ => {}
        }
    }
}

/// Analyze a CSS at-rule.
fn analyze_atrule(atrule: &CssAtrule, analysis: &mut CssAnalysis, in_global_block: bool) {
    // Check for @keyframes
    if atrule.name == "keyframes" || atrule.name == "-webkit-keyframes" {
        let keyframe_name = atrule.prelude.trim();

        if !keyframe_name.is_empty() {
            // -global- prefix means it's a global keyframe
            if keyframe_name.starts_with("-global-") {
                analysis.has_global = true;
            } else if !in_global_block {
                // Local keyframe - add to list for scoping
                analysis.keyframes.push(keyframe_name.to_string());
            }
        }
    }

    // Recurse into block children
    if let Some(block) = &atrule.block {
        for child in &block.children {
            match child {
                lux_ast::css::CssBlockChild::Rule(rule) => {
                    analyze_rule(rule, analysis, in_global_block);
                }
                lux_ast::css::CssBlockChild::Atrule(nested_atrule) => {
                    analyze_atrule(nested_atrule, analysis, in_global_block);
                }
                _ => {}
            }
        }
    }
}

/// Check if a selector list contains :global.
fn rule_has_global(prelude: &lux_ast::css::SelectorList) -> bool {
    prelude.children.iter().any(|complex| {
        complex.children.iter().any(|relative| {
            relative.selectors.iter().any(|sel| {
                matches!(sel, SimpleSelector::PseudoClassSelector(pseudo) if pseudo.name == "global")
            })
        })
    })
}
