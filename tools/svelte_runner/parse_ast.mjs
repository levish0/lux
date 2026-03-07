import { readFileSync, writeFileSync } from "node:fs";
import { parse } from "svelte/compiler";

const args = process.argv.slice(2);
const loose = args.includes("--loose");
const positional = args.filter((arg) => arg !== "--loose");
const [inputPath, outputPath] = positional;

if (!inputPath || !outputPath) {
  console.error("usage: node parse_ast.mjs [--loose] <input.svelte> <output.json>");
  process.exit(2);
}

const source = readFileSync(inputPath, "utf8").replace(/\r/g, "");
const ast = parse(source, { modern: true, loose });
writeFileSync(outputPath, JSON.stringify(ast));
