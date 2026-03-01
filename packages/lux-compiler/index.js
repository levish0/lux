import { createRequire } from "node:module";
import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const here = dirname(fileURLToPath(import.meta.url));

const candidates = [
  process.env.LUX_NODE_BINDING,
  resolve(here, "lux_node.node"),
].filter(Boolean);

let loaded = null;
for (const candidate of candidates) {
  if (!existsSync(candidate)) continue;
  loaded = require(candidate);
  break;
}

if (!loaded) {
  throw new Error(
    "Lux N-API binding not found. Run `npm run build:debug` or `npm run build:release` in packages/lux-compiler."
  );
}

export const compile = loaded.compile;
export const compileStrict = loaded.compileStrict;
