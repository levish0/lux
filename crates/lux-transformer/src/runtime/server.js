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
let props_id_counter = 0;

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
        normalized_key.startsWith("$$")
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
  props_id_counter += 1;
  return "lux-" + props_id_counter.toString(36);
}

export function begin_render() {
  return null;
}

export function end_render() {
  return { events: [], bindings: [], actions: [], transitions: [], animations: [] };
}

export function event_attr() {
  return "";
}

export function event_target_attr() {
  return "";
}

export function bind_attr() {
  return "";
}

export function bind_target_attr() {
  return "";
}

export function use_attr() {
  return "";
}

export function transition_attr() {
  return "";
}

export function animate_attr() {
  return "";
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

export function is_mount_target() {
  return false;
}

export function cleanup_mount() {}

export function mount_html() {}

export function mount_head() {}

export function mount_events() {}

export function mount_bindings() {}

export function mount_actions() {}

export function mount_transitions() {}

export function mount_animations() {}

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
