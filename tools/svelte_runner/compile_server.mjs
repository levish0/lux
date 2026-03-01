import { readFileSync, writeFileSync } from "node:fs";
import { compile } from "svelte/compiler";

const [, , inputPath, outputPath] = process.argv;

if (!inputPath || !outputPath) {
  console.error("usage: node compile_server.mjs <input.svelte> <output.json>");
  process.exit(2);
}

const source = readFileSync(inputPath, "utf8").replace(/\r/g, "");
const output = {
  js: null,
  css: null,
  warnings: [],
  error: null,
};

try {
  const result = compile(source, {
    filename: inputPath,
    generate: "server",
    modernAst: true,
  });
  output.js = result.js?.code ?? "";
  output.css = result.css?.code ?? null;
  output.warnings = (result.warnings ?? []).map((warning) => warning?.code ?? null);
} catch (error) {
  output.error = String(error?.message ?? error);
  output.warnings = Array.isArray(error?.warnings)
    ? error.warnings.map((warning) => warning?.code ?? null)
    : [];
}

writeFileSync(outputPath, JSON.stringify(output));
if (output.error) {
  process.exit(1);
}
