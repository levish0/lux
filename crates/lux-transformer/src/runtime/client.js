const BOOLEAN_ATTRIBUTE_NAMES = new Set([
  "allowfullscreen",
  "async",
  "autofocus",
  "autoplay",
  "checked",
  "controls",
  "default",
  "defer",
  "disabled",
  "disablepictureinpicture",
  "disableremoteplayback",
  "formnovalidate",
  "indeterminate",
  "inert",
  "ismap",
  "loop",
  "multiple",
  "muted",
  "nomodule",
  "novalidate",
  "open",
  "playsinline",
  "readonly",
  "required",
  "reversed",
  "seamless",
  "selected",
  "webkitdirectory"
]);
const INVALID_ATTR_NAME_CHAR_REGEX =
  /[\s'">/=\u{FDD0}-\u{FDEF}\u{FFFE}\u{FFFF}\u{1FFFE}\u{1FFFF}\u{2FFFE}\u{2FFFF}\u{3FFFE}\u{3FFFF}\u{4FFFE}\u{4FFFF}\u{5FFFE}\u{5FFFF}\u{6FFFE}\u{6FFFF}\u{7FFFE}\u{7FFFF}\u{8FFFE}\u{8FFFF}\u{9FFFE}\u{9FFFF}\u{AFFFE}\u{AFFFF}\u{BFFFE}\u{BFFFF}\u{CFFFE}\u{CFFFF}\u{DFFFE}\u{DFFFF}\u{EFFFE}\u{EFFFF}\u{FFFFE}\u{FFFFF}\u{10FFFE}\u{10FFFF}]/u;
let props_id_counter = 0;
const anchor_regions = new WeakMap();
const anchor_head_state = new WeakMap();
const anchor_mount_state = new WeakMap();
let current_render_state = null;

export function stringify(value) {
  if (typeof value === "string") return value;
  if (value == null) return "";
  return value + "";
}

export function escape(value) {
  return stringify(value).replace(/[&<>]/g, (ch) => {
    if (ch === "&") return "&amp;";
    if (ch === "<") return "&lt;";
    return "&gt;";
  });
}

export function escape_attr(value) {
  return stringify(value).replace(/[&<>"']/g, (ch) => {
    if (ch === "&") return "&amp;";
    if (ch === "<") return "&lt;";
    if (ch === ">") return "&gt;";
    if (ch === "\"") return "&quot;";
    return "&#39;";
  });
}

export function is_boolean_attr(name) {
  return BOOLEAN_ATTRIBUTE_NAMES.has(stringify(name).toLowerCase());
}

export function attr(name, value, is_boolean) {
  const normalized_name = stringify(name);
  const lower_name = normalized_name.toLowerCase();
  const effective_boolean =
    is_boolean || (lower_name === "hidden" && value !== "until-found");

  const normalized_value =
    lower_name === "translate" && value === true
      ? "yes"
      : lower_name === "translate" && value === false
      ? "no"
      : value;

  if (normalized_value == null || (effective_boolean && !normalized_value)) {
    return "";
  }

  if (effective_boolean) {
    return " " + normalized_name;
  }

  return " " + normalized_name + "=\"" + escape_attr(normalized_value) + "\"";
}

export function class_attr(base, toggles) {
  const tokens = [];
  push_class_tokens(tokens, base);
  if (toggles && typeof toggles === "object") {
    for (const [name, enabled] of Object.entries(toggles)) {
      if (enabled) {
        push_class_tokens(tokens, name);
      }
    }
  }
  if (tokens.length === 0) {
    return null;
  }
  return Array.from(new Set(tokens)).join(" ");
}

export function style_attr(base, styles) {
  const entries = [];

  if (base != null && base !== false) {
    const text = stringify(base).trim();
    if (text) {
      entries.push(strip_trailing_semicolon(text));
    }
  }

  if (styles && typeof styles === "object") {
    for (const [name, value] of Object.entries(styles)) {
      if (value == null || value === false) {
        continue;
      }
      const text = stringify(value).trim();
      if (!text) {
        continue;
      }
      entries.push(`${name}: ${text}`);
    }
  }

  if (entries.length === 0) {
    return null;
  }
  return entries.join("; ");
}

export function attributes(
  attrs,
  class_toggles = null,
  style_values = null,
  class_base = null,
  style_base = null
) {
  const source = attrs ?? {};
  const merged_class = class_attr(
    class_base != null ? class_base : source.class,
    class_toggles
  );
  const merged_style = style_attr(
    style_base != null ? style_base : source.style,
    style_values
  );
  const pairs = Object.entries(source);
  return pairs
    .map(([key, value]) => {
      const normalized_key = stringify(key);
      if (
        typeof value === "function" ||
        normalized_key === "class" ||
        normalized_key === "style" ||
        normalized_key.startsWith("$$") ||
        INVALID_ATTR_NAME_CHAR_REGEX.test(normalized_key)
      ) {
        return "";
      }
      return attr(normalized_key, value, is_boolean_attr(normalized_key));
    })
    .join("")
    .concat(
      attr("class", merged_class, false),
      attr("style", merged_style, false)
    );
}

export function props_id() {
  const root = typeof window === "undefined" ? globalThis : window;
  if (!root.__svelte) {
    root.__svelte = {};
  }
  if (typeof root.__svelte.uid !== "number") {
    root.__svelte.uid = 1;
  }
  return "c" + root.__svelte.uid++;
}

export function rest_props(props, exclude = []) {
  const source = props ?? {};
  const exclusions = new Set([
    "__lux_self",
    "$$slots",
    "$$events",
    "children",
    ...exclude.map((value) => stringify(value))
  ]);
  const result = {};
  for (const [key, value] of Object.entries(source)) {
    if (!exclusions.has(key)) {
      result[key] = value;
    }
  }
  return result;
}

export function begin_render() {
  const state = {
    events: [],
    bindings: [],
    actions: [],
    transitions: [],
    animations: [],
    parent: current_render_state
  };
  current_render_state = state;
  return state;
}

export function end_render(state) {
  if (state && current_render_state === state) {
    current_render_state = state.parent ?? null;
    return {
      events: state.events,
      bindings: state.bindings,
      actions: state.actions,
      transitions: state.transitions,
      animations: state.animations
    };
  }
  return {
    events: [],
    bindings: [],
    actions: [],
    transitions: [],
    animations: []
  };
}

export function render_component(component, props) {
  if (component && typeof component.render === "function") {
    return component.render(props ?? {});
  }
  if (typeof component === "function") {
    return component(props ?? {});
  }
  return "";
}

export function event_attr(name, handler, modifiers = []) {
  if (!current_render_state || typeof handler !== "function") {
    return "";
  }

  const event_name = normalize_event_name(name);
  if (!event_name) {
    return "";
  }

  const id = current_render_state.events.length;
  current_render_state.events.push({
    id,
    name: event_name,
    handler,
    modifiers: normalize_modifiers(modifiers)
  });

  return ` data-lux-on-${event_name}="${id}"`;
}

export function event_target_attr(target, name, handler, modifiers = []) {
  if (!current_render_state || typeof handler !== "function") {
    return "";
  }

  const target_name = normalize_mount_target_name(target);
  const event_name = normalize_event_name(name);
  if (!target_name || !event_name) {
    return "";
  }

  const id = current_render_state.events.length;
  current_render_state.events.push({
    id,
    target: target_name,
    name: event_name,
    handler,
    modifiers: normalize_modifiers(modifiers)
  });
  return "";
}

export function bind_attr(name, getter, setter) {
  if (!current_render_state || typeof setter !== "function") {
    return "";
  }

  const bind_name = normalize_bind_name(name);
  if (!bind_name) {
    return "";
  }

  const id = current_render_state.bindings.length;
  current_render_state.bindings.push({
    id,
    name: bind_name,
    getter: typeof getter === "function" ? getter : () => getter,
    setter
  });

  return ` data-lux-bind-${bind_name}="${id}"`;
}

export function bind_target_attr(target, name, getter, setter) {
  if (!current_render_state || typeof setter !== "function") {
    return "";
  }

  const target_name = normalize_mount_target_name(target);
  const bind_name = normalize_bind_name(name);
  if (!target_name || !bind_name) {
    return "";
  }

  const id = current_render_state.bindings.length;
  current_render_state.bindings.push({
    id,
    target: target_name,
    name: bind_name,
    getter: typeof getter === "function" ? getter : () => getter,
    setter
  });
  return "";
}

export function use_attr(name, action, parameter) {
  if (!current_render_state || typeof action !== "function") {
    return "";
  }

  const action_name = normalize_bind_name(name);
  if (!action_name) {
    return "";
  }

  const id = current_render_state.actions.length;
  current_render_state.actions.push({
    id,
    name: action_name,
    action,
    parameter
  });

  return ` data-lux-use-${action_name}="${id}"`;
}

export function transition_attr(name, transition, parameter, intro = true, outro = true) {
  if (!current_render_state || typeof transition !== "function") {
    return "";
  }

  const transition_name = normalize_bind_name(name);
  if (!transition_name) {
    return "";
  }

  const id = current_render_state.transitions.length;
  current_render_state.transitions.push({
    id,
    name: transition_name,
    transition,
    parameter,
    intro: Boolean(intro),
    outro: Boolean(outro)
  });

  return ` data-lux-transition-${transition_name}="${id}"`;
}

export function animate_attr(name, animation, parameter) {
  if (!current_render_state || typeof animation !== "function") {
    return "";
  }

  const animation_name = normalize_bind_name(name);
  if (!animation_name) {
    return "";
  }

  const id = current_render_state.animations.length;
  current_render_state.animations.push({
    id,
    name: animation_name,
    animation,
    parameter
  });

  return ` data-lux-animate-${animation_name}="${id}"`;
}

export function once(handler) {
  if (typeof handler !== "function") {
    return handler;
  }
  let called = false;
  return function once_wrapper(...args) {
    if (called) {
      return undefined;
    }
    called = true;
    return handler.apply(this, args);
  };
}

export function is_mount_target(value) {
  if (value == null) return false;
  if (typeof Node !== "undefined" && value instanceof Node) return true;
  return typeof value === "object" && typeof value.nodeType === "number";
}

export function cleanup_mount(anchor) {
  if (!is_mount_target(anchor)) return;

  clear_anchor_mount_state(anchor);
  clear_anchor_head_state(anchor);

  const region = anchor_regions.get(anchor);
  if (region && region.start?.parentNode && region.end?.parentNode) {
    clear_region(region);
    const parent = region.start.parentNode;
    if (parent && parent === region.end.parentNode) {
      parent.removeChild(region.start);
      parent.removeChild(region.end);
    }
    anchor_regions.delete(anchor);
    return;
  }

  if (anchor.nodeType === 1 || anchor.nodeType === 11 || anchor.nodeType === 9) {
    const host = anchor.nodeType === 9 ? anchor.body ?? anchor.documentElement : anchor;
    if (host && "innerHTML" in host) {
      host.innerHTML = "";
    }
  }
}

export function mount_html(anchor, html) {
  if (!is_mount_target(anchor)) return;
  clear_anchor_mount_state(anchor);

  const safe_html = stringify(html);

  if (anchor.nodeType === 1 || anchor.nodeType === 11 || anchor.nodeType === 9) {
    // Element, DocumentFragment, Document
    const host = anchor.nodeType === 9 ? anchor.body ?? anchor.documentElement : anchor;
    if (host && "innerHTML" in host) {
      host.innerHTML = safe_html;
      return;
    }
  }

  const owner_document =
    anchor.ownerDocument || (typeof document !== "undefined" ? document : null);
  const parent = anchor.parentNode;
  if (!owner_document || !parent) return;

  const region = ensure_anchor_region(anchor, owner_document);
  if (!region) return;

  clear_region(region);

  const template = owner_document.createElement("template");
  template.innerHTML = safe_html;
  parent.insertBefore(template.content, region.end);
}

export function mount_head(anchor, html) {
  if (!is_mount_target(anchor)) return;

  clear_anchor_head_state(anchor);

  const owner_document = resolve_owner_document(anchor);
  const head = owner_document?.head ?? null;
  if (!owner_document || !head) {
    return;
  }

  const template = owner_document.createElement("template");
  template.innerHTML = stringify(html);
  const nodes = Array.from(template.content.childNodes);
  if (nodes.length === 0) {
    return;
  }

  head.appendChild(template.content);
  anchor_head_state.set(anchor, { head, nodes });
}

export function mount_events(anchor, events) {
  if (!is_mount_target(anchor)) {
    return;
  }

  const mount_state = ensure_anchor_mount_state(anchor);
  run_cleanup_list(mount_state.event_cleanups);
  mount_state.event_cleanups = [];

  if (!Array.isArray(events) || events.length === 0) {
    return;
  }

  const event_targets = resolve_mount_event_targets(anchor);
  if (event_targets.length === 0) {
    return;
  }

  for (const event of events) {
    if (!event || typeof event.name !== "string" || typeof event.id !== "number") {
      continue;
    }

    if (event.target) {
      const cleanup = mount_global_event(event);
      if (typeof cleanup === "function") {
        mount_state.event_cleanups.push(cleanup);
      }
      continue;
    }

    const attr_name = `data-lux-on-${event.name}`;
    const attr_value = String(event.id);

    for (const root of event_targets) {
      bind_event_for_root(root, attr_name, attr_value, event, mount_state.event_cleanups);
    }
  }
}

export function mount_bindings(anchor, bindings) {
  if (!is_mount_target(anchor)) {
    return;
  }

  const mount_state = ensure_anchor_mount_state(anchor);
  run_cleanup_list(mount_state.binding_cleanups);
  mount_state.binding_cleanups = [];

  if (!Array.isArray(bindings) || bindings.length === 0) {
    return;
  }

  const binding_targets = resolve_mount_event_targets(anchor);
  if (binding_targets.length === 0) {
    return;
  }

  const group_buckets = new Map();

  for (const binding of bindings) {
    if (!binding || typeof binding.name !== "string" || typeof binding.id !== "number") {
      continue;
    }

    if (binding.target) {
      const cleanup = mount_global_binding(binding);
      if (typeof cleanup === "function") {
        mount_state.binding_cleanups.push(cleanup);
      }
      continue;
    }

    const attr_name = `data-lux-bind-${binding.name}`;
    const attr_value = String(binding.id);
    const elements = [];
    for (const root of binding_targets) {
      collect_binding_elements(root, attr_name, attr_value, elements);
    }

    if (elements.length === 0) {
      continue;
    }

    if (binding.name === "group") {
      const key = get_group_binding_key(binding);
      let bucket = group_buckets.get(key);
      if (!bucket) {
        bucket = {
          binding,
          attr_name,
          elements: new Set()
        };
        group_buckets.set(key, bucket);
      }
      for (const element of elements) {
        bucket.elements.add(element);
      }
      continue;
    }

    for (const element of elements) {
      const cleanup = apply_binding(element, attr_name, binding);
      if (typeof cleanup === "function") {
        mount_state.binding_cleanups.push(cleanup);
      }
    }
  }

  for (const bucket of group_buckets.values()) {
    const cleanup = apply_group_binding(
      Array.from(bucket.elements),
      bucket.attr_name,
      bucket.binding
    );
    if (typeof cleanup === "function") {
      mount_state.binding_cleanups.push(cleanup);
    }
  }
}

export function mount_actions(anchor, actions) {
  if (!is_mount_target(anchor)) {
    return;
  }

  const mount_state = ensure_anchor_mount_state(anchor);
  run_cleanup_list(mount_state.action_cleanups);
  mount_state.action_cleanups = [];

  if (!Array.isArray(actions) || actions.length === 0) {
    return;
  }

  const action_targets = resolve_mount_event_targets(anchor);
  if (action_targets.length === 0) {
    return;
  }

  for (const action of actions) {
    if (!action || typeof action.name !== "string" || typeof action.id !== "number") {
      continue;
    }

    const attr_name = `data-lux-use-${action.name}`;
    const attr_value = String(action.id);
    for (const root of action_targets) {
      mount_action_for_root(root, attr_name, attr_value, action, mount_state.action_cleanups);
    }
  }
}

export function mount_transitions(anchor, transitions) {
  if (!is_mount_target(anchor)) {
    return;
  }

  const mount_state = ensure_anchor_mount_state(anchor);
  run_cleanup_list(mount_state.transition_cleanups);
  mount_state.transition_cleanups = [];

  if (!Array.isArray(transitions) || transitions.length === 0) {
    return;
  }

  const transition_targets = resolve_mount_event_targets(anchor);
  if (transition_targets.length === 0) {
    return;
  }

  for (const transition of transitions) {
    if (!transition || typeof transition.name !== "string" || typeof transition.id !== "number") {
      continue;
    }

    const attr_name = `data-lux-transition-${transition.name}`;
    const attr_value = String(transition.id);
    for (const root of transition_targets) {
      mount_transition_for_root(
        root,
        attr_name,
        attr_value,
        transition,
        mount_state.transition_cleanups
      );
    }
  }
}

export function mount_animations(anchor, animations) {
  if (!is_mount_target(anchor)) {
    return;
  }

  const mount_state = ensure_anchor_mount_state(anchor);
  run_cleanup_list(mount_state.animation_cleanups);
  mount_state.animation_cleanups = [];

  if (!Array.isArray(animations) || animations.length === 0) {
    return;
  }

  const animation_targets = resolve_mount_event_targets(anchor);
  if (animation_targets.length === 0) {
    return;
  }

  for (const animation of animations) {
    if (!animation || typeof animation.name !== "string" || typeof animation.id !== "number") {
      continue;
    }

    const attr_name = `data-lux-animate-${animation.name}`;
    const attr_value = String(animation.id);
    for (const root of animation_targets) {
      mount_animation_for_root(
        root,
        attr_name,
        attr_value,
        animation,
        mount_state.animation_cleanups
      );
    }
  }
}

function ensure_anchor_mount_state(anchor) {
  const existing = anchor_mount_state.get(anchor);
  if (existing) {
    return existing;
  }
  const state = {
    event_cleanups: [],
    binding_cleanups: [],
    action_cleanups: [],
    transition_cleanups: [],
    animation_cleanups: []
  };
  anchor_mount_state.set(anchor, state);
  return state;
}

function clear_anchor_mount_state(anchor) {
  const state = anchor_mount_state.get(anchor);
  if (!state) {
    return;
  }
  run_cleanup_list(state.event_cleanups);
  run_cleanup_list(state.binding_cleanups);
  run_cleanup_list(state.action_cleanups);
  run_cleanup_list(state.transition_cleanups);
  run_cleanup_list(state.animation_cleanups);
  anchor_mount_state.delete(anchor);
}

function clear_anchor_head_state(anchor) {
  const state = anchor_head_state.get(anchor);
  if (!state) {
    return;
  }
  for (const node of state.nodes) {
    if (node?.parentNode) {
      node.parentNode.removeChild(node);
    }
  }
  anchor_head_state.delete(anchor);
}

function run_cleanup_list(cleanups) {
  if (!Array.isArray(cleanups) || cleanups.length === 0) {
    return;
  }
  for (const cleanup of cleanups) {
    if (typeof cleanup !== "function") {
      continue;
    }
    try {
      cleanup();
    } catch {}
  }
}

function ensure_anchor_region(anchor, owner_document) {
  const existing = anchor_regions.get(anchor);
  if (existing && existing.start?.parentNode && existing.end?.parentNode) {
    return existing;
  }

  const parent = anchor.parentNode;
  if (!parent) return null;

  const start = owner_document.createComment("lux-start");
  const end = owner_document.createComment("lux-end");
  parent.insertBefore(start, anchor);
  parent.insertBefore(end, anchor);

  const region = { start, end };
  anchor_regions.set(anchor, region);
  return region;
}

function clear_region(region) {
  const { start, end } = region;
  const parent = start.parentNode;
  if (!parent || parent !== end.parentNode) return;

  let node = start.nextSibling;
  while (node && node !== end) {
    const next = node.nextSibling;
    parent.removeChild(node);
    node = next;
  }
}

function resolve_owner_document(anchor) {
  if (anchor?.nodeType === 9) {
    return anchor;
  }
  return anchor?.ownerDocument || (typeof document !== "undefined" ? document : null);
}

function resolve_mount_event_targets(anchor) {
  const region = anchor_regions.get(anchor);
  if (region && region.start?.parentNode && region.end?.parentNode) {
    return collect_region_roots(region);
  }

  if (anchor.nodeType === 9) {
    return anchor.body ? [anchor.body] : [];
  }
  if (anchor.nodeType === 1 || anchor.nodeType === 11) {
    return [anchor];
  }

  return [];
}

function collect_region_roots(region) {
  const roots = [];
  let node = region.start.nextSibling;
  while (node && node !== region.end) {
    if (node.nodeType === 1) {
      roots.push(node);
    }
    node = node.nextSibling;
  }
  return roots;
}

function bind_event_for_root(root, attr_name, attr_value, event, cleanups) {
  if (root.nodeType !== 1) {
    return;
  }

  if (root.getAttribute(attr_name) === attr_value) {
    const cleanup = attach_event(root, attr_name, event);
    if (typeof cleanup === "function") {
      cleanups.push(cleanup);
    }
  }

  const selector = `[${attr_name}="${css_escape(attr_value)}"]`;
  const nodes = root.querySelectorAll(selector);
  for (const node of nodes) {
    const cleanup = attach_event(node, attr_name, event);
    if (typeof cleanup === "function") {
      cleanups.push(cleanup);
    }
  }
}

function mount_action_for_root(root, attr_name, attr_value, action, cleanups) {
  if (root.nodeType !== 1) {
    return;
  }

  if (root.getAttribute(attr_name) === attr_value) {
    const cleanup = apply_action(root, attr_name, action);
    if (typeof cleanup === "function") {
      cleanups.push(cleanup);
    }
  }

  const selector = `[${attr_name}="${css_escape(attr_value)}"]`;
  const nodes = root.querySelectorAll(selector);
  for (const node of nodes) {
    const cleanup = apply_action(node, attr_name, action);
    if (typeof cleanup === "function") {
      cleanups.push(cleanup);
    }
  }
}

function mount_transition_for_root(root, attr_name, attr_value, transition, cleanups) {
  if (root.nodeType !== 1) {
    return;
  }

  if (root.getAttribute(attr_name) === attr_value) {
    const cleanup = apply_transition(root, attr_name, transition);
    if (typeof cleanup === "function") {
      cleanups.push(cleanup);
    }
  }

  const selector = `[${attr_name}="${css_escape(attr_value)}"]`;
  const nodes = root.querySelectorAll(selector);
  for (const node of nodes) {
    const cleanup = apply_transition(node, attr_name, transition);
    if (typeof cleanup === "function") {
      cleanups.push(cleanup);
    }
  }
}

function mount_animation_for_root(root, attr_name, attr_value, animation, cleanups) {
  if (root.nodeType !== 1) {
    return;
  }

  if (root.getAttribute(attr_name) === attr_value) {
    const cleanup = apply_animation(root, attr_name, animation);
    if (typeof cleanup === "function") {
      cleanups.push(cleanup);
    }
  }

  const selector = `[${attr_name}="${css_escape(attr_value)}"]`;
  const nodes = root.querySelectorAll(selector);
  for (const node of nodes) {
    const cleanup = apply_animation(node, attr_name, animation);
    if (typeof cleanup === "function") {
      cleanups.push(cleanup);
    }
  }
}

function collect_binding_elements(root, attr_name, attr_value, out) {
  if (root.nodeType !== 1) {
    return;
  }

  if (root.getAttribute(attr_name) === attr_value) {
    out.push(root);
  }

  const selector = `[${attr_name}="${css_escape(attr_value)}"]`;
  const nodes = root.querySelectorAll(selector);
  for (const node of nodes) {
    out.push(node);
  }
}

function mount_global_event(event) {
  const target = resolve_global_target(event.target);
  if (!target || typeof target.addEventListener !== "function") {
    return undefined;
  }
  const listener = build_event_listener(event, target);
  const options = build_event_listener_options(event.modifiers);
  target.addEventListener(event.name, listener, options);
  return () => {
    target.removeEventListener(event.name, listener, options);
  };
}

function mount_global_binding(binding) {
  const target_name = normalize_mount_target_name(binding.target);
  const target = resolve_global_target(target_name);
  if (!target) {
    return undefined;
  }

  if (target_name === "body") {
    return apply_binding(target, "", binding);
  }

  if (typeof binding.setter !== "function") {
    return undefined;
  }

  const sync = () => {
    binding.setter(read_global_binding_value(target_name, target, binding.name));
  };

  switch (binding.name) {
    case "online": {
      sync();
      const window_target = resolve_global_target("window");
      if (!window_target || typeof window_target.addEventListener !== "function") {
        return undefined;
      }
      window_target.addEventListener("online", sync);
      window_target.addEventListener("offline", sync);
      return () => {
        window_target.removeEventListener("online", sync);
        window_target.removeEventListener("offline", sync);
      };
    }
    case "scrollx":
    case "scrolly": {
      if (typeof binding.getter === "function" && typeof target.scrollTo === "function") {
        const current = Number(binding.getter());
        if (Number.isFinite(current)) {
          if (binding.name === "scrollx") {
            target.scrollTo(current, target.scrollY ?? 0);
          } else {
            target.scrollTo(target.scrollX ?? 0, current);
          }
        }
      }
      sync();
      target.addEventListener("scroll", sync, { passive: true });
      return () => {
        target.removeEventListener("scroll", sync, { passive: true });
      };
    }
    case "innerwidth":
    case "innerheight":
    case "outerwidth":
    case "outerheight":
    case "devicepixelratio": {
      sync();
      target.addEventListener("resize", sync, { passive: true });
      return () => {
        target.removeEventListener("resize", sync, { passive: true });
      };
    }
    case "activeelement": {
      sync();
      target.addEventListener("focusin", sync);
      target.addEventListener("focusout", sync);
      return () => {
        target.removeEventListener("focusin", sync);
        target.removeEventListener("focusout", sync);
      };
    }
    case "fullscreenelement": {
      sync();
      target.addEventListener("fullscreenchange", sync);
      return () => {
        target.removeEventListener("fullscreenchange", sync);
      };
    }
    case "pointerlockelement": {
      sync();
      target.addEventListener("pointerlockchange", sync);
      return () => {
        target.removeEventListener("pointerlockchange", sync);
      };
    }
    case "visibilitystate": {
      sync();
      target.addEventListener("visibilitychange", sync);
      return () => {
        target.removeEventListener("visibilitychange", sync);
      };
    }
    default: {
      sync();
      return undefined;
    }
  }
}

function read_global_binding_value(target_name, target, binding_name) {
  switch (binding_name) {
    case "online":
      return target?.navigator?.onLine ?? false;
    case "scrollx":
      return target?.scrollX ?? 0;
    case "scrolly":
      return target?.scrollY ?? 0;
    case "innerwidth":
      return target?.innerWidth ?? 0;
    case "innerheight":
      return target?.innerHeight ?? 0;
    case "outerwidth":
      return target?.outerWidth ?? 0;
    case "outerheight":
      return target?.outerHeight ?? 0;
    case "devicepixelratio":
      return target?.devicePixelRatio ?? 1;
    case "activeelement":
      return target?.activeElement ?? null;
    case "fullscreenelement":
      return target?.fullscreenElement ?? null;
    case "pointerlockelement":
      return target?.pointerLockElement ?? null;
    case "visibilitystate":
      return target?.visibilityState ?? "visible";
    default:
      return target?.[binding_name];
  }
}

function attach_event(element, attr_name, event) {
  const listener = build_event_listener(event, element);
  const options = build_event_listener_options(event.modifiers);
  element.addEventListener(event.name, listener, options);
  remove_binding_marker(element, attr_name);
  return () => {
    element.removeEventListener(event.name, listener, options);
  };
}

function build_event_listener(event, element) {
  const modifiers = event.modifiers;
  const handler = event.handler;

  return (dom_event) => {
    if (modifiers.self && dom_event.target !== element) {
      return;
    }
    if (modifiers.trusted && !dom_event.isTrusted) {
      return;
    }
    if (modifiers.stopPropagation) {
      dom_event.stopPropagation();
    }
    if (modifiers.stopImmediatePropagation) {
      dom_event.stopImmediatePropagation();
    }
    if (modifiers.preventDefault) {
      dom_event.preventDefault();
    }
    handler.call(element, dom_event);
  };
}

function build_event_listener_options(modifiers) {
  const options = {};
  if (modifiers.capture) options.capture = true;
  if (modifiers.once) options.once = true;
  if (modifiers.passive) options.passive = true;
  if (modifiers.nonpassive) options.passive = false;
  return Object.keys(options).length === 0 ? false : options;
}

function normalize_event_name(name) {
  const normalized = stringify(name).trim().toLowerCase();
  const cleaned = normalized.replace(/[^a-z0-9_-]/g, "");
  return cleaned || "";
}

function normalize_mount_target_name(target) {
  const normalized = stringify(target).trim().toLowerCase();
  if (normalized === "window" || normalized === "document" || normalized === "body") {
    return normalized;
  }
  return "";
}

function normalize_modifiers(modifiers) {
  const normalized = {
    capture: false,
    nonpassive: false,
    once: false,
    passive: false,
    preventDefault: false,
    self: false,
    stopImmediatePropagation: false,
    stopPropagation: false,
    trusted: false
  };

  if (!Array.isArray(modifiers)) {
    return normalized;
  }

  for (const modifier of modifiers) {
    const key = String(modifier);
    if (key in normalized) {
      normalized[key] = true;
    }
  }
  return normalized;
}

function css_escape(value) {
  if (typeof CSS !== "undefined" && typeof CSS.escape === "function") {
    return CSS.escape(value);
  }
  return String(value).replace(/["\\]/g, "\\$&");
}

function normalize_bind_name(name) {
  const normalized = stringify(name).trim().toLowerCase();
  const cleaned = normalized.replace(/[^a-z0-9_-]/g, "");
  return cleaned || "";
}

function resolve_global_target(target_name) {
  if (target_name === "window") {
    return typeof window !== "undefined" ? window : null;
  }
  if (target_name === "document") {
    return typeof document !== "undefined" ? document : null;
  }
  if (target_name === "body") {
    if (typeof document === "undefined") {
      return null;
    }
    return document.body ?? null;
  }
  return null;
}

function apply_binding(element, attr_name, binding) {
  const kind = binding.name;
  const normalized_kind = String(kind).toLowerCase();
  const getter = binding.getter;
  const setter = binding.setter;
  if (typeof setter !== "function") {
    remove_binding_marker(element, attr_name);
    return undefined;
  }

  if (normalized_kind === "this") {
    setter(element);
    remove_binding_marker(element, attr_name);
    return () => setter(null);
  }

  if (normalized_kind === "value") {
    if (is_select_element(element)) {
      const current = typeof getter === "function" ? getter() : undefined;
      if (current === undefined) {
        if (element.multiple) {
          setter(Array.from(element.selectedOptions, (option) => option.value));
        } else {
          const selected =
            element.querySelector(":checked") ??
            element.querySelector("option:not([disabled])");
          setter(selected ? selected.value : element.value);
        }
      } else {
        set_select_value(element, current);
      }

      const listener = () => {
        if (element.multiple) {
          setter(Array.from(element.selectedOptions, (option) => option.value));
        } else {
          setter(element.value);
        }
      };
      element.addEventListener("change", listener);
      remove_binding_marker(element, attr_name);
      return () => {
        element.removeEventListener("change", listener);
      };
    }

    set_element_value(element, getter());
    const listener = (event) => {
      setter(event?.currentTarget?.value ?? "");
    };
    element.addEventListener("input", listener);
    element.addEventListener("change", listener);
    remove_binding_marker(element, attr_name);
    return () => {
      element.removeEventListener("input", listener);
      element.removeEventListener("change", listener);
    };
  }

  if (normalized_kind === "checked") {
    set_element_checked(element, getter());
    const listener = (event) => {
      setter(Boolean(event?.currentTarget?.checked));
    };
    element.addEventListener("change", listener);
    remove_binding_marker(element, attr_name);
    return () => {
      element.removeEventListener("change", listener);
    };
  }

  if (normalized_kind === "files") {
    set_element_files(element, getter());
    const listener = (event) => {
      setter(event?.currentTarget?.files ?? null);
    };
    element.addEventListener("change", listener);
    remove_binding_marker(element, attr_name);
    return () => {
      element.removeEventListener("change", listener);
    };
  }

  if (normalized_kind === "open") {
    set_element_open(element, getter());
    const listener = (event) => {
      setter(Boolean(event?.currentTarget?.open));
    };
    element.addEventListener("toggle", listener);
    remove_binding_marker(element, attr_name);
    return () => {
      element.removeEventListener("toggle", listener);
    };
  }

  if (normalized_kind === "indeterminate") {
    set_element_indeterminate(element, getter());
    const listener = (event) => {
      setter(Boolean(event?.currentTarget?.indeterminate));
    };
    element.addEventListener("change", listener);
    remove_binding_marker(element, attr_name);
    return () => {
      element.removeEventListener("change", listener);
    };
  }

  if (
    normalized_kind === "innertext" ||
    normalized_kind === "innerhtml" ||
    normalized_kind === "textcontent"
  ) {
    set_element_text_property(element, normalized_kind, getter());
    const listener = () => {
      setter(read_element_text_property(element, normalized_kind));
    };
    element.addEventListener("input", listener);
    remove_binding_marker(element, attr_name);
    return () => {
      element.removeEventListener("input", listener);
    };
  }

  if (normalized_kind === "focused") {
    const cleanup = apply_focused_binding(element, setter, getter);
    remove_binding_marker(element, attr_name);
    return cleanup;
  }

  if (is_media_bind_kind(normalized_kind)) {
    const cleanup = apply_media_binding(element, normalized_kind, setter, getter);
    remove_binding_marker(element, attr_name);
    return cleanup;
  }

  if (
    normalized_kind === "clientwidth" ||
    normalized_kind === "clientheight" ||
    normalized_kind === "offsetwidth" ||
    normalized_kind === "offsetheight"
  ) {
    const cleanup = apply_size_binding(element, normalized_kind, setter);
    remove_binding_marker(element, attr_name);
    return cleanup;
  }

  if (
    normalized_kind === "contentrect" ||
    normalized_kind === "contentboxsize" ||
    normalized_kind === "borderboxsize" ||
    normalized_kind === "devicepixelcontentboxsize"
  ) {
    const cleanup = apply_resize_observer_binding(element, normalized_kind, setter);
    remove_binding_marker(element, attr_name);
    return cleanup;
  }

  if (normalized_kind === "naturalwidth" || normalized_kind === "naturalheight") {
    const cleanup = apply_readonly_property_binding(
      element,
      normalized_kind,
      setter,
      ["load"]
    );
    remove_binding_marker(element, attr_name);
    return cleanup;
  }

  // Fallback: treat unknown bind as value-like binding.
  set_element_value(element, getter());
  const listener = (event) => {
    setter(event?.currentTarget?.value ?? "");
  };
  element.addEventListener("change", listener);
  remove_binding_marker(element, attr_name);
  return () => {
    element.removeEventListener("change", listener);
  };
}

function apply_action(element, attr_name, descriptor) {
  if (!descriptor || typeof descriptor.action !== "function") {
    remove_binding_marker(element, attr_name);
    return undefined;
  }

  const parameter =
    typeof descriptor.parameter === "function"
      ? descriptor.parameter()
      : descriptor.parameter;

  let result;
  try {
    result = descriptor.action(element, parameter);
  } catch {
    remove_binding_marker(element, attr_name);
    return undefined;
  }

  remove_binding_marker(element, attr_name);

  if (typeof result === "function") {
    return result;
  }
  if (result && typeof result.destroy === "function") {
    return () => {
      result.destroy();
    };
  }
  return undefined;
}

function apply_transition(element, attr_name, descriptor) {
  if (!descriptor || typeof descriptor.transition !== "function") {
    remove_binding_marker(element, attr_name);
    return undefined;
  }

  const parameter =
    typeof descriptor.parameter === "function"
      ? descriptor.parameter()
      : descriptor.parameter;

  const options = {
    direction:
      descriptor.intro && descriptor.outro
        ? "both"
        : descriptor.intro
        ? "in"
        : descriptor.outro
        ? "out"
        : "both"
  };

  let result;
  try {
    result = descriptor.transition(element, parameter, options);
  } catch {
    remove_binding_marker(element, attr_name);
    return undefined;
  }

  if (descriptor.intro && result && typeof result.in === "function") {
    try {
      result.in();
    } catch {}
  }

  remove_binding_marker(element, attr_name);

  return () => {
    if (descriptor.outro && result && typeof result.out === "function") {
      try {
        result.out();
      } catch {}
    }
    if (typeof result === "function") {
      result();
      return;
    }
    if (result && typeof result.destroy === "function") {
      result.destroy();
    }
  };
}

function apply_animation(element, attr_name, descriptor) {
  if (!descriptor || typeof descriptor.animation !== "function") {
    remove_binding_marker(element, attr_name);
    return undefined;
  }

  const parameter =
    typeof descriptor.parameter === "function"
      ? descriptor.parameter()
      : descriptor.parameter;

  let result;
  try {
    result = descriptor.animation(element, parameter);
  } catch {
    remove_binding_marker(element, attr_name);
    return undefined;
  }

  remove_binding_marker(element, attr_name);

  return () => {
    if (typeof result === "function") {
      result();
      return;
    }
    if (result && typeof result.destroy === "function") {
      result.destroy();
    }
  };
}

function set_element_value(element, value) {
  if (!("value" in element)) {
    return;
  }
  const next = value == null ? "" : stringify(value);
  if (element.value !== next) {
    element.value = next;
  }
}

function set_element_checked(element, value) {
  if (!("checked" in element)) {
    return;
  }
  element.checked = Boolean(value);
}

function set_element_open(element, value) {
  if (!("open" in element)) {
    return;
  }
  element.open = Boolean(value);
}

function set_element_indeterminate(element, value) {
  if (!("indeterminate" in element)) {
    return;
  }
  element.indeterminate = Boolean(value);
}

function set_element_text_property(element, property, value) {
  const target_property = normalize_text_property_name(property);
  if (!(target_property in element)) {
    return;
  }
  const next = value == null ? "" : stringify(value);
  if (element[target_property] !== next) {
    element[target_property] = next;
  }
}

function read_element_text_property(element, property) {
  const target_property = normalize_text_property_name(property);
  if (!(target_property in element)) {
    return "";
  }
  return element[target_property];
}

function set_element_files(element, files) {
  if (!("files" in element)) {
    return;
  }
  try {
    element.files = files ?? null;
  } catch {
    // Browsers can reject programmatic FileList assignment.
  }
}

function is_select_element(element) {
  return (
    element &&
    typeof element.tagName === "string" &&
    element.tagName.toLowerCase() === "select"
  );
}

function set_select_value(select, value) {
  if (select.multiple) {
    if (!Array.isArray(value)) {
      return;
    }

    const selected = new Set(value.map((item) => String(item)));
    for (const option of select.options) {
      option.selected = selected.has(option.value);
    }
    return;
  }

  let matched = false;
  for (const option of select.options) {
    const selected = binding_value_equals(option.value, value);
    option.selected = selected;
    matched = matched || selected;
  }
  if (!matched && value !== undefined) {
    select.selectedIndex = -1;
  }
}

function normalize_text_property_name(property) {
  const normalized = String(property).toLowerCase();
  if (normalized === "innertext") return "innerText";
  if (normalized === "innerhtml") return "innerHTML";
  return "textContent";
}

function apply_focused_binding(element, setter, getter) {
  const sync = () => setter(resolve_focused_binding_value(element));
  if (typeof getter === "function") {
    const desired = Boolean(getter());
    if (desired && typeof element.focus === "function") {
      element.focus();
    }
    if (!desired && typeof element.blur === "function") {
      element.blur();
    }
  }
  element.addEventListener("focus", sync);
  element.addEventListener("blur", sync);
  sync();
  return () => {
    element.removeEventListener("focus", sync);
    element.removeEventListener("blur", sync);
    setter(false);
  };
}

function resolve_focused_binding_value(element) {
  if (typeof document === "undefined") {
    return false;
  }
  return document.activeElement === element;
}

function is_media_bind_kind(kind) {
  return (
    kind === "currenttime" ||
    kind === "duration" ||
    kind === "paused" ||
    kind === "buffered" ||
    kind === "seekable" ||
    kind === "played" ||
    kind === "volume" ||
    kind === "muted" ||
    kind === "playbackrate" ||
    kind === "seeking" ||
    kind === "ended" ||
    kind === "readystate" ||
    kind === "videoheight" ||
    kind === "videowidth"
  );
}

function apply_media_binding(element, kind, setter, getter) {
  const events = media_bind_events(kind);
  const writable = media_bind_is_writable(kind);
  if (writable && typeof getter === "function") {
    const next = getter();
    write_media_property(element, kind, next);
  }
  const sync = () => setter(read_media_property(element, kind));
  for (const event_name of events) {
    element.addEventListener(event_name, sync);
  }
  sync();
  return () => {
    for (const event_name of events) {
      element.removeEventListener(event_name, sync);
    }
  };
}

function media_bind_events(kind) {
  switch (kind) {
    case "currenttime":
      return ["timeupdate", "seeking", "seeked"];
    case "duration":
      return ["durationchange", "loadedmetadata"];
    case "paused":
      return ["play", "pause", "ended"];
    case "buffered":
      return ["progress", "loadedmetadata", "durationchange"];
    case "seekable":
      return ["progress", "loadedmetadata", "durationchange"];
    case "played":
      return ["timeupdate", "play", "pause", "ended"];
    case "volume":
    case "muted":
      return ["volumechange"];
    case "playbackrate":
      return ["ratechange"];
    case "seeking":
      return ["seeking", "seeked", "timeupdate"];
    case "ended":
      return ["ended", "timeupdate"];
    case "readystate":
      return ["loadeddata", "loadedmetadata", "canplay", "canplaythrough"];
    case "videoheight":
    case "videowidth":
      return ["resize", "loadedmetadata", "loadeddata"];
    default:
      return ["change"];
  }
}

function media_bind_is_writable(kind) {
  return (
    kind === "currenttime" ||
    kind === "paused" ||
    kind === "volume" ||
    kind === "muted" ||
    kind === "playbackrate"
  );
}

function read_media_property(element, kind) {
  switch (kind) {
    case "currenttime":
      return element.currentTime;
    case "duration":
      return element.duration;
    case "paused":
      return element.paused;
    case "buffered":
      return element.buffered;
    case "seekable":
      return element.seekable;
    case "played":
      return element.played;
    case "volume":
      return element.volume;
    case "muted":
      return element.muted;
    case "playbackrate":
      return element.playbackRate;
    case "seeking":
      return element.seeking;
    case "ended":
      return element.ended;
    case "readystate":
      return element.readyState;
    case "videoheight":
      return element.videoHeight;
    case "videowidth":
      return element.videoWidth;
    default:
      return undefined;
  }
}

function write_media_property(element, kind, value) {
  switch (kind) {
    case "currenttime": {
      const next = Number(value);
      if (Number.isFinite(next)) {
        element.currentTime = next;
      }
      return;
    }
    case "paused": {
      const should_pause = Boolean(value);
      if (should_pause) {
        if (typeof element.pause === "function") {
          element.pause();
        }
      } else if (typeof element.play === "function") {
        const result = element.play();
        if (result && typeof result.catch === "function") {
          result.catch(() => {});
        }
      }
      return;
    }
    case "volume": {
      const next = Number(value);
      if (Number.isFinite(next)) {
        element.volume = next;
      }
      return;
    }
    case "muted":
      element.muted = Boolean(value);
      return;
    case "playbackrate": {
      const next = Number(value);
      if (Number.isFinite(next)) {
        element.playbackRate = next;
      }
      return;
    }
    default:
      return;
  }
}

function apply_size_binding(element, kind, setter) {
  const sync = () => setter(read_size_property(element, kind));
  const resize_options = { passive: true };
  if (typeof window !== "undefined") {
    window.addEventListener("resize", sync, resize_options);
  }
  const resize_observer_cleanup = observe_resize(element, () => sync());
  sync();
  return () => {
    if (typeof window !== "undefined") {
      window.removeEventListener("resize", sync, resize_options);
    }
    resize_observer_cleanup();
  };
}

function read_size_property(element, kind) {
  if (kind === "clientwidth") return element.clientWidth;
  if (kind === "clientheight") return element.clientHeight;
  if (kind === "offsetwidth") return element.offsetWidth;
  return element.offsetHeight;
}

function apply_resize_observer_binding(element, kind, setter) {
  const sync_fallback = () => {
    if (kind === "contentrect") {
      setter(element.getBoundingClientRect?.() ?? null);
      return;
    }
    setter(undefined);
  };

  if (typeof ResizeObserver !== "function") {
    const resize_options = { passive: true };
    if (typeof window !== "undefined") {
      window.addEventListener("resize", sync_fallback, resize_options);
    }
    sync_fallback();
    return () => {
      if (typeof window !== "undefined") {
        window.removeEventListener("resize", sync_fallback, resize_options);
      }
    };
  }

  const observer = new ResizeObserver((entries) => {
    for (const entry of entries) {
      if (entry.target !== element) {
        continue;
      }
      if (kind === "contentrect") {
        setter(entry.contentRect);
        continue;
      }
      if (kind === "contentboxsize") {
        setter(entry.contentBoxSize);
        continue;
      }
      if (kind === "borderboxsize") {
        setter(entry.borderBoxSize);
        continue;
      }
      setter(entry.devicePixelContentBoxSize);
    }
  });
  observer.observe(element);
  sync_fallback();
  return () => {
    observer.disconnect();
  };
}

function observe_resize(element, callback) {
  if (typeof ResizeObserver !== "function") {
    return () => {};
  }
  const observer = new ResizeObserver(() => callback());
  observer.observe(element);
  return () => {
    observer.disconnect();
  };
}

function apply_readonly_property_binding(element, kind, setter, events) {
  const sync = () => setter(read_readonly_property(element, kind));
  for (const event_name of events) {
    element.addEventListener(event_name, sync);
  }
  sync();
  return () => {
    for (const event_name of events) {
      element.removeEventListener(event_name, sync);
    }
  };
}

function read_readonly_property(element, kind) {
  if (kind === "naturalwidth") return element.naturalWidth;
  return element.naturalHeight;
}

function apply_group_binding(elements, attr_name, binding) {
  if (!Array.isArray(elements) || elements.length === 0) {
    return undefined;
  }

  const setter = binding.setter;
  const getter = binding.getter;
  if (typeof setter !== "function") {
    for (const element of elements) {
      remove_binding_marker(element, attr_name);
    }
    return undefined;
  }

  const has_checkbox = elements.some((element) => is_checkbox_input(element));
  const sync_from_model = () => {
    const current = typeof getter === "function" ? getter() : undefined;

    if (has_checkbox) {
      const list = Array.isArray(current) ? current : [];
      for (const element of elements) {
        element.checked = list.some((value) =>
          binding_value_equals(value, read_group_input_value(element))
        );
      }
      return;
    }

    for (const element of elements) {
      if (is_radio_input(element)) {
        element.checked = binding_value_equals(read_group_input_value(element), current);
      }
    }
  };

  const update_model = (source) => {
    if (has_checkbox) {
      const next = [];
      for (const element of elements) {
        if (element.checked) {
          next.push(read_group_input_value(element));
        }
      }
      setter(next);
      return;
    }

    if (source?.checked) {
      setter(read_group_input_value(source));
    }
  };

  const listeners = [];
  for (const element of elements) {
    const listener = () => update_model(element);
    listeners.push([element, listener]);
    element.addEventListener("change", listener);
    remove_binding_marker(element, attr_name);
  }

  sync_from_model();
  return () => {
    for (const [element, listener] of listeners) {
      element.removeEventListener("change", listener);
    }
  };
}

function read_group_input_value(element) {
  if (element && "__value" in element) {
    return element.__value;
  }
  if (element && "value" in element) {
    return element.value;
  }
  return element?.getAttribute?.("value");
}

function is_checkbox_input(element) {
  return (
    element &&
    typeof element.tagName === "string" &&
    element.tagName.toLowerCase() === "input" &&
    String(element.type).toLowerCase() === "checkbox"
  );
}

function is_radio_input(element) {
  return (
    element &&
    typeof element.tagName === "string" &&
    element.tagName.toLowerCase() === "input" &&
    String(element.type).toLowerCase() === "radio"
  );
}

function binding_value_equals(left, right) {
  return Object.is(left, right) || String(left) === String(right);
}

function get_group_binding_key(binding) {
  if (binding && binding.groupKey != null) {
    return `key:${String(binding.groupKey)}`;
  }
  if (typeof binding.setter === "function") {
    return `setter:${binding.setter.toString()}`;
  }
  if (typeof binding.getter === "function") {
    return `getter:${binding.getter.toString()}`;
  }
  return `id:${String(binding?.id ?? "")}`;
}

function remove_binding_marker(element, attr_name) {
  if (!attr_name || typeof element?.removeAttribute !== "function") {
    return;
  }
  element.removeAttribute(attr_name);
}

function push_class_tokens(out, value) {
  if (value == null || value === false) {
    return;
  }
  const text = stringify(value).trim();
  if (!text) {
    return;
  }
  for (const token of text.split(/\s+/)) {
    if (token) {
      out.push(token);
    }
  }
}

function strip_trailing_semicolon(text) {
  const trimmed = text.trim();
  if (!trimmed.endsWith(";")) {
    return trimmed;
  }
  return trimmed.slice(0, -1).trimEnd();
}
