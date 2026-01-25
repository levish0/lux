import fs from 'node:fs';

const svelte = JSON.parse(fs.readFileSync('ToParse/output/svelte.json'));
const lux = JSON.parse(fs.readFileSync('ToParse/output/lux.json'));

// Focus on fragment nodes (each blocks)
console.log("=== Each Block Comparison ===\n");

const svelteEach = svelte.fragment.nodes.filter(n => n.type === 'EachBlock');
const luxEach = lux.fragment.nodes.filter(n => n.type === 'EachBlock');

for (let i = 0; i < svelteEach.length; i++) {
  console.log("Each Block " + (i + 1) + ":");
  console.log("  Svelte context: start=" + svelteEach[i].context.start + ", end=" + svelteEach[i].context.end + ", type=" + svelteEach[i].context.type);
  console.log("  Lux context:    start=" + luxEach[i].context.start + ", end=" + luxEach[i].context.end + ", type=" + luxEach[i].context.type);

  if (svelteEach[i].context.typeAnnotation) {
    console.log("  Svelte typeAnnotation: start=" + svelteEach[i].context.typeAnnotation.start + ", end=" + svelteEach[i].context.typeAnnotation.end);
  }
  if (luxEach[i].context.typeAnnotation) {
    console.log("  Lux typeAnnotation:    start=" + luxEach[i].context.typeAnnotation.start + ", end=" + luxEach[i].context.typeAnnotation.end);
  }
  console.log();
}
