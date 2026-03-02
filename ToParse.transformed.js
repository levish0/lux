import { stringify as __lux_stringify, escape as __lux_escape, escape_attr as __lux_escape_attr, attr as __lux_attr, attributes as __lux_attributes, is_boolean_attr as __lux_is_boolean_attr } from "lux/runtime/server";
const __lux_template = "";
const __lux_css = null;
const __lux_css_hash = null;
const __lux_css_scope = null;
const __lux_has_dynamic = true;
export { __lux_template as template, __lux_css as css, __lux_css_hash as cssHash, __lux_css_scope as cssScope, __lux_has_dynamic as hasDynamic };
export default {
	template: __lux_template,
	css: __lux_css,
	cssHash: __lux_css_hash,
	cssScope: __lux_css_scope,
	hasDynamic: __lux_has_dynamic,
	render: function __lux_render(_props = {}) {
		_props.__lux_self == null && (_props.__lux_self = __lux_render);
		let { value = undefined } = _props;
		return function() {
			let __lux_chunks = [];
			__lux_chunks.push(__lux_escape(__lux_stringify(value)));
			return __lux_chunks.join("");
		}();
	}
};
