const __lux_template = "\n\n<button>clicks: </button>\n";
const __lux_css = null;
const __lux_css_hash = null;
const __lux_css_scope = null;
const __lux_has_dynamic = true;
const __lux_stringify = function(value) {
	return typeof value === "string" ? value : value == null ? "" : value + "";
};
const __lux_escape = function(value) {
	return __lux_stringify(value).replace(/[&<>]/g, function(ch) {
		return ch === "&" ? "&amp;" : ch === "<" ? "&lt;" : "&gt;";
	});
};
const __lux_escape_attr = function(value) {
	return __lux_stringify(value).replace(/[&<>"']/g, function(ch) {
		return ch === "&" ? "&amp;" : ch === "<" ? "&lt;" : ch === ">" ? "&gt;" : ch === "\"" ? "&quot;" : "&#39;";
	});
};
const __lux_is_boolean_attr = function(name) {
	return [
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
	].includes(__lux_stringify(name).toLowerCase());
};
const __lux_attr = function(name, value, is_boolean) {
	return value == null || (is_boolean || __lux_stringify(name).toLowerCase() === "hidden" && value !== "until-found") && !value ? "" : is_boolean || __lux_stringify(name).toLowerCase() === "hidden" && value !== "until-found" ? " " + __lux_stringify(name) : " " + __lux_stringify(name) + "=\"" + __lux_escape_attr(__lux_stringify(name).toLowerCase() === "translate" && value === true ? "yes" : __lux_stringify(name).toLowerCase() === "translate" && value === false ? "no" : value) + "\"";
};
const __lux_attributes = function(attrs) {
	return Object.entries(attrs ?? {}).map(function(__lux_entry) {
		return typeof __lux_entry[1] === "function" || __lux_stringify(__lux_entry[0]).startsWith("$$") ? "" : __lux_attr(__lux_stringify(__lux_entry[0]), __lux_entry[1], __lux_is_boolean_attr(__lux_stringify(__lux_entry[0])));
	}).join("");
};
export { __lux_template as template, __lux_css as css, __lux_css_hash as cssHash, __lux_css_scope as cssScope, __lux_has_dynamic as hasDynamic };
export default {
	template: __lux_template,
	css: __lux_css,
	cssHash: __lux_css_hash,
	cssScope: __lux_css_scope,
	hasDynamic: __lux_has_dynamic,
	render: function __lux_render(_props = {}) {
		_props.__lux_self == null && (_props.__lux_self = __lux_render);
		let count = 0;
		return function() {
			let __lux_chunks = [];
			__lux_chunks.push("\n\n");
			__lux_chunks.push([
				"<button",
				"",
				">",
				function() {
					let __lux_chunks = [];
					__lux_chunks.push("clicks: ");
					__lux_chunks.push(__lux_escape(__lux_stringify(count)));
					return __lux_chunks.join("");
				}(),
				"</button>"
			].join(""));
			__lux_chunks.push("\n");
			return __lux_chunks.join("");
		}();
	}
};
