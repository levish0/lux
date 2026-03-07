use oxc_allocator::Vec as ArenaVec;
use oxc_ast::{
    AstBuilder, NONE,
    ast::{Expression, Statement, VariableDeclarationKind},
};
use oxc_span::SPAN;

pub(super) const LUX_TEMPLATE: &str = "__lux_template";
pub(super) const LUX_CSS: &str = "__lux_css";
pub(super) const LUX_CSS_HASH: &str = "__lux_css_hash";
pub(super) const LUX_CSS_SCOPE: &str = "__lux_css_scope";
pub(super) const LUX_HAS_DYNAMIC: &str = "__lux_has_dynamic";

pub(super) const LUX_STRINGIFY: &str = "__lux_stringify";
pub(super) const LUX_ESCAPE: &str = "__lux_escape";
pub(super) const LUX_ESCAPE_ATTR: &str = "__lux_escape_attr";
pub(super) const LUX_ATTR: &str = "__lux_attr";
pub(super) const LUX_CLASS_ATTR: &str = "__lux_class_attr";
pub(super) const LUX_STYLE_ATTR: &str = "__lux_style_attr";
pub(super) const LUX_ATTRIBUTES: &str = "__lux_attributes";
pub(super) const LUX_IS_BOOLEAN_ATTR: &str = "__lux_is_boolean_attr";
pub(super) const LUX_PROPS_ID: &str = "__lux_props_id";
pub(super) const LUX_MOUNT_HTML: &str = "__lux_mount_html";
pub(super) const LUX_MOUNT_HEAD: &str = "__lux_mount_head";
pub(super) const LUX_IS_MOUNT_TARGET: &str = "__lux_is_mount_target";
pub(super) const LUX_CLEANUP_MOUNT: &str = "__lux_cleanup_mount";
pub(super) const LUX_BEGIN_RENDER: &str = "__lux_begin_render";
pub(super) const LUX_END_RENDER: &str = "__lux_end_render";
pub(super) const LUX_EVENT_ATTR: &str = "__lux_event_attr";
pub(super) const LUX_EVENT_TARGET_ATTR: &str = "__lux_event_target_attr";
pub(super) const LUX_MOUNT_EVENTS: &str = "__lux_mount_events";
pub(super) const LUX_ONCE: &str = "__lux_once";
pub(super) const LUX_BIND_ATTR: &str = "__lux_bind_attr";
pub(super) const LUX_BIND_TARGET_ATTR: &str = "__lux_bind_target_attr";
pub(super) const LUX_MOUNT_BINDINGS: &str = "__lux_mount_bindings";
pub(super) const LUX_USE_ATTR: &str = "__lux_use_attr";
pub(super) const LUX_MOUNT_ACTIONS: &str = "__lux_mount_actions";
pub(super) const LUX_TRANSITION_ATTR: &str = "__lux_transition_attr";
pub(super) const LUX_MOUNT_TRANSITIONS: &str = "__lux_mount_transitions";
pub(super) const LUX_ANIMATE_ATTR: &str = "__lux_animate_attr";
pub(super) const LUX_MOUNT_ANIMATIONS: &str = "__lux_mount_animations";

pub(super) const LUX_RUNTIME_SERVER_IMPORT_SOURCE: &str = "import { stringify as __lux_stringify, escape as __lux_escape, escape_attr as __lux_escape_attr, attr as __lux_attr, class_attr as __lux_class_attr, style_attr as __lux_style_attr, attributes as __lux_attributes, is_boolean_attr as __lux_is_boolean_attr, props_id as __lux_props_id, mount_html as __lux_mount_html, mount_head as __lux_mount_head, is_mount_target as __lux_is_mount_target, cleanup_mount as __lux_cleanup_mount, begin_render as __lux_begin_render, end_render as __lux_end_render, event_attr as __lux_event_attr, event_target_attr as __lux_event_target_attr, mount_events as __lux_mount_events, once as __lux_once, bind_attr as __lux_bind_attr, bind_target_attr as __lux_bind_target_attr, mount_bindings as __lux_mount_bindings, use_attr as __lux_use_attr, mount_actions as __lux_mount_actions, transition_attr as __lux_transition_attr, mount_transitions as __lux_mount_transitions, animate_attr as __lux_animate_attr, mount_animations as __lux_mount_animations } from \"lux/runtime/server\";";
pub(super) const LUX_RUNTIME_CLIENT_IMPORT_SOURCE: &str = "import { stringify as __lux_stringify, escape as __lux_escape, escape_attr as __lux_escape_attr, attr as __lux_attr, class_attr as __lux_class_attr, style_attr as __lux_style_attr, attributes as __lux_attributes, is_boolean_attr as __lux_is_boolean_attr, props_id as __lux_props_id, mount_html as __lux_mount_html, mount_head as __lux_mount_head, is_mount_target as __lux_is_mount_target, cleanup_mount as __lux_cleanup_mount, begin_render as __lux_begin_render, end_render as __lux_end_render, event_attr as __lux_event_attr, event_target_attr as __lux_event_target_attr, mount_events as __lux_mount_events, once as __lux_once, bind_attr as __lux_bind_attr, bind_target_attr as __lux_bind_target_attr, mount_bindings as __lux_mount_bindings, use_attr as __lux_use_attr, mount_actions as __lux_mount_actions, transition_attr as __lux_transition_attr, mount_transitions as __lux_mount_transitions, animate_attr as __lux_animate_attr, mount_animations as __lux_mount_animations } from \"lux/runtime/client\";";

pub(super) fn push_const<'a>(
    ast: AstBuilder<'a>,
    body: &mut ArenaVec<'a, Statement<'a>>,
    name: &str,
    init: Expression<'a>,
) {
    let declarator = ast.variable_declarator(
        SPAN,
        VariableDeclarationKind::Const,
        ast.binding_pattern_binding_identifier(SPAN, ast.ident(name)),
        NONE,
        Some(init),
        false,
    );
    let declaration = ast.declaration_variable(
        SPAN,
        VariableDeclarationKind::Const,
        ast.vec1(declarator),
        false,
    );
    body.push(declaration.into());
}

pub(super) fn optional_string_expr<'a>(ast: AstBuilder<'a>, value: Option<&str>) -> Expression<'a> {
    value.map_or_else(
        || ast.expression_null_literal(SPAN),
        |value| ast.expression_string_literal(SPAN, ast.atom(value), None),
    )
}
