import { parse } from 'svelte/compiler';
import fs from 'node:fs';
import path from 'node:path';

const testsDir = path.resolve(import.meta.dirname, 'svelte@5.48.0-tests');
const outputDir = path.resolve(import.meta.dirname, 'output');

if (!fs.existsSync(testsDir)) {
  console.error(`Reference tests not found: ${testsDir}`);
  process.exit(1);
}

fs.mkdirSync(outputDir, { recursive: true });

const SKIP_CATEGORIES = ['parser-legacy'];

// Recursively find all .svelte files
function findSvelteFiles(dir) {
  const results = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      results.push(...findSvelteFiles(full));
    } else if (entry.name.endsWith('.svelte')) {
      results.push(full);
    }
  }
  return results;
}

const allFiles = findSvelteFiles(testsDir);
const files = allFiles.filter(f => {
  const rel = path.relative(testsDir, f).replace(/\\/g, '/');
  return !SKIP_CATEGORIES.some(cat => rel.startsWith(cat + '/'));
});
console.log(`Found ${files.length} .svelte files (skipped ${allFiles.length - files.length} from: ${SKIP_CATEGORIES.join(', ')})`);

let okCount = 0;
let failCount = 0;

for (const file of files) {
  // Build name from relative path: category--testname--filename
  const rel = path.relative(testsDir, file);
  const parts = rel.replace(/\\/g, '/').split('/');
  // Remove "samples" from path parts
  const filtered = parts.filter(p => p !== 'samples');
  // Remove .svelte extension from last part
  filtered[filtered.length - 1] = filtered[filtered.length - 1].replace('.svelte', '');
  const name = filtered.join('--');

  const source = fs.readFileSync(file, 'utf-8')
    .replace(/\r\n/g, '\n')
    .replace(/\s+$/, '');

  const loose = name.includes('loose-');

  try {
    const ast = JSON.parse(JSON.stringify(
      parse(source, { modern: true, loose })
    ));
    delete ast.comments;

    const outDir = path.join(outputDir, name);
    fs.mkdirSync(outDir, { recursive: true });
    fs.writeFileSync(
      path.join(outDir, 'svelte.json'),
      JSON.stringify(ast, null, '\t') + '\n'
    );
    okCount++;
  } catch (e) {
    const outDir = path.join(outputDir, name);
    fs.mkdirSync(outDir, { recursive: true });
    fs.writeFileSync(
      path.join(outDir, 'svelte-error.txt'),
      e.message + '\n'
    );
    failCount++;
  }
}

console.log(`Done: ${okCount} OK, ${failCount} parse errors`);
