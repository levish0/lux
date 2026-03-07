import { getAllContexts } from "svelte";
import { render as svelteRender } from "svelte/server";

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
const TITLE_REGEX = /<title(?:\s[^>]*)?>[\s\S]*?<\/title>/gi;
const VOID_TAG_NAME_REGEX =
  /<(area|base|br|col|embed|hr|img|input|link|meta|param|source|track|wbr)\b([^>]*)>/gi;

export function stringify(value) {
  if (typeof value === "string") return value;
  if (value == null) return "";
  return value + "";
}

export function escape(value) {
  return stringify(value).replace(/[&<]/g, (ch) => {
    if (ch === "&") return "&amp;";
    return "&lt;";
  });
}

export function escape_attr(value) {
  return stringify(value).replace(/[&<"]/g, (ch) => {
    if (ch === "&") return "&amp;";
    if (ch === "<") return "&lt;";
    return "&quot;";
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
  const renderer = arguments[0];
  if (
    renderer &&
    renderer.global &&
    typeof renderer.global.uid === "function" &&
    typeof renderer.push === "function"
  ) {
    const uid = renderer.global.uid();
    renderer.push("<!--$" + uid + "-->");
    return uid;
  }

  props_id_counter += 1;
  return "s" + props_id_counter.toString(36);
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

export function finalize_head(value) {
  const html = stringify(value);
  if (!html) {
    return "";
  }

  let last_title = "";
  const without_titles = html.replace(TITLE_REGEX, (match) => {
    last_title = match;
    return "";
  });
  const compact = self_close_void_tags(
    without_titles.replace(/>\s+</g, "> <").trim()
  );

  return last_title ? compact + last_title : compact;
}

export function store_get(store_values, store_name, store) {
  if (
    store_values &&
    store_name in store_values &&
    store_values[store_name][0] === store
  ) {
    return store_values[store_name][2];
  }

  store_values?.[store_name]?.[1]?.();

  let value;
  let unsubscribe = () => {};
  if (store && typeof store.subscribe === "function") {
    const result = store.subscribe((next) => {
      value = next;
    });
    unsubscribe =
      typeof result === "function"
        ? result
        : typeof result?.unsubscribe === "function"
        ? result.unsubscribe.bind(result)
        : () => {};
  }

  if (store_values) {
    store_values[store_name] = [store, unsubscribe, value];
  }
  return value;
}

export function unsubscribe_stores(store_values) {
  if (!store_values) {
    return;
  }
  for (const store_name in store_values) {
    store_values[store_name]?.[1]?.();
  }
}

export function begin_render() {
  return { head: "" };
}

export function end_render(state) {
  return {
    head: state?.head ?? "",
    events: [],
    bindings: [],
    actions: [],
    transitions: [],
    animations: []
  };
}

export function render_component(component, props, render_state = null) {
  if (typeof component === "function") {
    const options = { props: props ?? {} };
    const context = current_render_context();
    if (context) {
      options.context = context;
    }
    const result = svelteRender(component, options);
    const body = result?.body ?? result?.html ?? "";
    if (render_state && result?.head) {
      render_state.head += result.head;
    }
    return body;
  }

  if (component && typeof component.render === "function") {
    const normalized_props = props ?? {};
    const body = component.render(normalized_props) ?? "";
    const head =
      typeof component.head === "function" ? component.head(normalized_props) ?? "" : "";
    if (render_state && head) {
      render_state.head += head;
    }
    return body;
  }

  return "";
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

function self_close_void_tags(html) {
  return html.replace(VOID_TAG_NAME_REGEX, (match, tag, attrs) => {
    const trimmed_attrs = attrs ?? "";
    if (trimmed_attrs.trimEnd().endsWith("/")) {
      return match;
    }
    return `<${tag}${trimmed_attrs}/>`;
  });
}

function current_render_context() {
  try {
    return new Map(getAllContexts());
  } catch {
    return null;
  }
}
