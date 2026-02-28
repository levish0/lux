const __lux_template = "\n\n<h1>Pick a colour</h1>\n\n<div>\n  <button style=\"background: red\" aria-label=\"red\"></button>\n\n  <button style=\"background: orange\" aria-label=\"orange\"></button>\n\n  <button style=\"background: yellow\" aria-label=\"yellow\"></button>\n\n  <!-- TODO add the rest of the colours -->\n  <button></button>\n  <button></button>\n  <button></button>\n  <button></button>\n</div>\n\n\n";
const __lux_css = "h1.svelte-ew55e7 { font-size: 2rem; font-weight: 700; transition: color 0.2s; }\ndiv.svelte-ew55e7 { display: grid; grid-template-columns: repeat(7, 1fr); grid-gap: 5px; max-width: 400px; }\nbutton.svelte-ew55e7 { aspect-ratio: 1; border-radius: 50%; background: var(--color, #fff); transform: translate(-2px,-2px); filter: drop-shadow(2px 2px 3px rgba(0,0,0,0.2)); transition: all 0.1s; color: black; font-weight: 700; font-size: 2rem; }\nbutton[aria-current=true].svelte-ew55e7 { transform: none; filter: none; box-shadow: inset 3px 3px 4px rgba(0,0,0,0.2); }";
const __lux_css_hash = "ew55e7";
const __lux_css_scope = "svelte-ew55e7";
const __lux_has_dynamic = true;
export { __lux_template as template, __lux_css as css, __lux_css_hash as cssHash, __lux_css_scope as cssScope, __lux_has_dynamic as hasDynamic };
export default {
	template: __lux_template,
	css: __lux_css,
	cssHash: __lux_css_hash,
	cssScope: __lux_css_scope,
	hasDynamic: __lux_has_dynamic,
	render: function(_props = {}) {
		return "" + "\n\n" + ("<h1" + (" style=\"" + ("" + "color: " + String(function({ selected }) {
			return selected;
		}(_props) ?? "").replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll("\"", "&quot;").replaceAll("'", "&#39;")) + "\"") + ">" + ("" + "Pick a colour") + "</h1>") + "\n\n" + ("<div" + ">" + ("" + "\n  " + ("<button" + (" style=\"" + ("" + "background: red") + "\"") + (" aria-label=\"" + ("" + "red") + "\"") + (" aria-current=\"" + String(function({ selected }) {
			return selected === "red";
		}(_props) ?? "").replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll("\"", "&quot;").replaceAll("'", "&#39;") + "\"") + (" onclick=\"" + String(function({ selected }) {
			return () => selected = "red";
		}(_props) ?? "").replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll("\"", "&quot;").replaceAll("'", "&#39;") + "\"") + ">" + "" + "</button>") + "\n\n  " + ("<button" + (" style=\"" + ("" + "background: orange") + "\"") + (" aria-label=\"" + ("" + "orange") + "\"") + (" aria-current=\"" + String(function({ selected }) {
			return selected === "orange";
		}(_props) ?? "").replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll("\"", "&quot;").replaceAll("'", "&#39;") + "\"") + (" onclick=\"" + String(function({ selected }) {
			return () => selected = "orange";
		}(_props) ?? "").replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll("\"", "&quot;").replaceAll("'", "&#39;") + "\"") + ">" + "" + "</button>") + "\n\n  " + ("<button" + (" style=\"" + ("" + "background: yellow") + "\"") + (" aria-label=\"" + ("" + "yellow") + "\"") + (" aria-current=\"" + String(function({ selected }) {
			return selected === "yellow";
		}(_props) ?? "").replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll("\"", "&quot;").replaceAll("'", "&#39;") + "\"") + (" onclick=\"" + String(function({ selected }) {
			return () => selected = "yellow";
		}(_props) ?? "").replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll("\"", "&quot;").replaceAll("'", "&#39;") + "\"") + ">" + "" + "</button>") + "\n\n  " + "<!-- TODO add the rest of the colours -->" + "\n  " + ("<button" + ">" + "" + "</button>") + "\n  " + ("<button" + ">" + "" + "</button>") + "\n  " + ("<button" + ">" + "" + "</button>") + "\n  " + ("<button" + ">" + "" + "</button>") + "\n") + "</div>") + "\n\n" + "\n";
	}
};
