import { readFileSync, writeFileSync } from "node:fs";
import { compile } from "svelte/compiler";

const [, , inputPath, outputPath] = process.argv;

if (!inputPath || !outputPath) {
  console.error(
    "usage: node analyze_diagnostics.mjs <input.svelte> <output.json>",
  );
  process.exit(2);
}

const source = readFileSync(inputPath, "utf8").replace(/\r/g, "");

const normalize = (diagnostic) => ({
  code: diagnostic?.code ?? null,
  start: diagnostic?.start?.character ?? null,
  end: diagnostic?.end?.character ?? null,
  message: String(diagnostic?.message ?? "").split("\n")[0],
});

const output = {
  errors: [],
  warnings: [],
};

try {
  const result = compile(source, {
    filename: inputPath,
    generate: false,
    modernAst: true,
  });
  output.warnings = result.warnings.map(normalize);
} catch (error) {
  output.errors = [normalize(error)];
  if (Array.isArray(error?.warnings)) {
    output.warnings = error.warnings.map(normalize);
  }
}

writeFileSync(outputPath, JSON.stringify(output));
