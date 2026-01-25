//! RenderTag visitor for analysis.
//!
//! Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/RenderTag.js`
//!
//! ```js
//! export function RenderTag(node, context) {
//!     validate_opening_tag(node, context.state, '@');
//!     node.metadata.path = [...context.path];
//!     context.state.analysis.uses_render_tags = true;
//!     // ... validation logic
//!     mark_subtree_dynamic(context.path);
//! }
//! ```

use lux_ast::tags::RenderTag;
use oxc_ast::ast::{Argument, Expression};

use crate::analyze::errors;
use crate::analyze::state::AnalysisState;
use crate::analyze::visitor::NodeKind;
use crate::analyze::visitors::shared::validate_opening_tag;

/// RenderTag visitor.
///
/// Reference: `packages/svelte/src/compiler/phases/2-analyze/visitors/RenderTag.js`
pub fn visit_render_tag(
    node: &RenderTag<'_>,
    state: &mut AnalysisState<'_, '_>,
    _path: &[NodeKind<'_>],
) {
    // validate_opening_tag(node, context.state, '@');
    // Note: RenderTag always validates (no runes condition)
    validate_opening_tag(node.span.into(), "@", state);

    // context.state.analysis.uses_render_tags = true;
    state.analysis.uses_render_tags = true;

    // const expression = unwrap_optional(node.expression);
    // Handle both CallExpression and ChainExpression (for optional calls like snippet?.())
    let call = match &node.expression {
        Expression::CallExpression(call) => Some(call.as_ref()),
        Expression::ChainExpression(chain) => {
            if let oxc_ast::ast::ChainElement::CallExpression(call) = &chain.expression {
                Some(call.as_ref())
            } else {
                None
            }
        }
        _ => None,
    };

    let Some(call) = call else {
        return;
    };

    // const raw_args = unwrap_optional(node.expression).arguments;
    // for (const arg of raw_args) {
    //     if (arg.type === 'SpreadElement') {
    //         e.render_tag_invalid_spread_argument(arg);
    //     }
    // }
    for arg in &call.arguments {
        if let Argument::SpreadElement(spread) = arg {
            state
                .analysis
                .error(errors::render_tag_invalid_spread_argument(spread.span));
        }
    }

    // if (
    //     callee.type === 'MemberExpression' &&
    //     callee.property.type === 'Identifier' &&
    //     ['bind', 'apply', 'call'].includes(callee.property.name)
    // ) {
    //     e.render_tag_invalid_call_expression(node);
    // }
    let callee = &call.callee;
    match callee {
        Expression::StaticMemberExpression(member) => {
            let prop_name = member.property.name.as_str();
            if prop_name == "bind" || prop_name == "apply" || prop_name == "call" {
                state
                    .analysis
                    .error(errors::render_tag_invalid_call_expression(node.span.into()));
            }
        }
        Expression::ComputedMemberExpression(member) => {
            // Check if property is an identifier
            if let Expression::Identifier(id) = &member.expression {
                let prop_name = id.name.as_str();
                if prop_name == "bind" || prop_name == "apply" || prop_name == "call" {
                    state
                        .analysis
                        .error(errors::render_tag_invalid_call_expression(node.span.into()));
                }
            }
        }
        _ => {}
    }

    // The rest of the reference (binding resolution, metadata updates, context.visit)
    // is handled by the main visitor traversal and scope analysis
}
