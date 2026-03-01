import { readFileSync } from "node:fs";
import { compile, parse } from "svelte/compiler";

const [, , phase, inputPath, iterationsArg] = process.argv;

if (!phase || !inputPath || !iterationsArg) {
  console.error(
    "usage: node benchmark_phase.mjs <parse|analyze|transform> <input.svelte> <iterations>",
  );
  process.exit(2);
}

if (phase !== "parse" && phase !== "analyze" && phase !== "transform") {
  console.error(`unsupported phase: ${phase}`);
  process.exit(2);
}

const iterations = Number.parseInt(iterationsArg, 10);
if (!Number.isFinite(iterations) || iterations < 1) {
  console.error(`invalid iterations: ${iterationsArg}`);
  process.exit(2);
}

const source = readFileSync(inputPath, "utf8").replace(/\r/g, "");
let sink = 0;

for (let i = 0; i < 3; i += 1) {
  if (phase === "parse") {
    const ast = parse(source, { modern: true });
    sink ^= ast.fragment.nodes.length;
  } else if (phase === "analyze") {
    const result = compile(source, {
      filename: inputPath,
      generate: false,
      modernAst: true,
    });
    sink ^= result.warnings.length;
  } else {
    const result = compile(source, {
      filename: inputPath,
      generate: "server",
      modernAst: true,
    });
    sink ^= result.js.code.length;
  }
}

const start = process.hrtime.bigint();
for (let i = 0; i < iterations; i += 1) {
  if (phase === "parse") {
    const ast = parse(source, { modern: true });
    sink ^= ast.fragment.nodes.length;
  } else if (phase === "analyze") {
    const result = compile(source, {
      filename: inputPath,
      generate: false,
      modernAst: true,
    });
    sink ^= result.warnings.length;
  } else {
    const result = compile(source, {
      filename: inputPath,
      generate: "server",
      modernAst: true,
    });
    sink ^= result.js.code.length;
  }
}
const elapsedNs = process.hrtime.bigint() - start;

process.stdout.write(`${elapsedNs} ${sink}\n`);
