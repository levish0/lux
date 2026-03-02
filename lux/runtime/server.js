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

export function attributes(attrs) {
  const pairs = Object.entries(attrs ?? {});
  return pairs
    .map(([key, value]) => {
      if (typeof value === "function" || stringify(key).startsWith("$$")) {
        return "";
      }
      const normalized_key = stringify(key);
      return attr(normalized_key, value, is_boolean_attr(normalized_key));
    })
    .join("");
}
