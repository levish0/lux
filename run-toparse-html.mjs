import fs from 'node:fs/promises';
import path from 'node:path';
import comp from './ToParse.transformed.js';

const html = comp.render();
const outPath = path.resolve('ToParse.html');
await fs.writeFile(outPath, html, 'utf8');
console.log(`Rendered HTML written to ${outPath}`);
