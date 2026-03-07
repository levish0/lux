import { readFileSync, writeFileSync } from "node:fs";
import vm from "node:vm";

const [, , inputPath, outputPath] = process.argv;

if (!inputPath || !outputPath) {
  console.error(
    "usage: node read_server_sample_config.mjs <_config.js> <output.json>",
  );
  process.exit(2);
}

const raw = readFileSync(inputPath, "utf8").replace(/\r/g, "");
const expression = raw
  .replace(/^\s*import\s+.*?;\s*$/gm, "")
  .replace(/export\s+default\s+test\s*\(/, "(")
  .trim();

const context = {
  test: (config) => config,
};

let config = {};
try {
  config = vm.runInNewContext(expression, context, { filename: inputPath }) ?? {};
} catch (error) {
  console.error(`failed to evaluate ${inputPath}: ${error?.message ?? error}`);
  process.exit(1);
}

writeFileSync(
  outputPath,
  JSON.stringify({
    props: config.props ?? {},
    id_prefix: config.id_prefix ?? null,
    csp: config.csp ?? null,
    mode: config.mode ?? null,
    error: config.error ?? null,
    compile_options: config.compileOptions ?? {},
    without_normalize_html: config.withoutNormalizeHtml ?? false,
    load_compiled: config.load_compiled ?? false,
    script_hashes: config.script_hashes ?? null,
  }),
);
