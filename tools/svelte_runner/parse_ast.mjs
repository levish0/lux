import { readFileSync, writeFileSync } from "node:fs";
import { parse } from "svelte/compiler";

const [, , inputPath, outputPath] = process.argv;

if (!inputPath || !outputPath) {
  console.error("usage: node parse_ast.mjs <input.svelte> <output.json>");
  process.exit(2);
}

const source = readFileSync(inputPath, "utf8").replace(/\r/g, "");
const ast = parse(source, { modern: true });
writeFileSync(outputPath, JSON.stringify(ast));
