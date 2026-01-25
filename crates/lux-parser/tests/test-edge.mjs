import { parse } from 'svelte/compiler';
import fs from 'node:fs';

const source = fs.readFileSync('ToParse/edge-cases.svelte', 'utf-8').replace(/\r\n/g, '\n');

try {
  const ast = JSON.parse(JSON.stringify(parse(source, { modern: true })));
  delete ast.comments;
  fs.mkdirSync('ToParse/output', { recursive: true });
  fs.writeFileSync('ToParse/output/svelte.json', JSON.stringify(ast, null, 2));
  console.log('Svelte parse OK');
} catch (e) {
  console.log('Svelte parse error:', e.message);
}
