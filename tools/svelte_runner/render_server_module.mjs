import { readFileSync, writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { render } from "svelte/server";

const [, , modulePath, renderConfigPath, outputPath] = process.argv;

if (!modulePath || !renderConfigPath || !outputPath) {
  console.error(
    "usage: node render_server_module.mjs <module.js> <render-config.json> <output.json>",
  );
  process.exit(2);
}

const renderConfig = JSON.parse(readFileSync(renderConfigPath, "utf8"));
const props = renderConfig.props ?? {};
const mod = await import(pathToFileURL(modulePath).href);
const component = mod.default;

let body = "";
let head = "";

if (component && typeof component.render === "function") {
  body = component.render(props, {
    idPrefix: renderConfig.id_prefix ?? undefined,
    csp: renderConfig.csp ?? undefined,
  }) ?? "";
  head = typeof component.head === "function"
    ? component.head(props, {
        idPrefix: renderConfig.id_prefix ?? undefined,
        csp: renderConfig.csp ?? undefined,
      }) ?? ""
    : "";
} else {
  const result = await render(component, {
    props,
    idPrefix: renderConfig.id_prefix ?? undefined,
    csp: renderConfig.csp ?? undefined,
  });
  body = result?.body ?? result?.html ?? "";
  head = result?.head ?? "";
}

writeFileSync(outputPath, JSON.stringify({ body, head }));
