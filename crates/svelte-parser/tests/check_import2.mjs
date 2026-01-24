import fs from 'fs';
import path from 'path';

const outDir = 'crates/svelte-parser/tests/output';
const samples = fs.readdirSync(outDir).filter(s => {
  const d = path.join(outDir, s);
  return fs.existsSync(path.join(d, 'lux.json')) && fs.existsSync(path.join(d, 'svelte.json'));
});

function findNodes(obj, type, results = []) {
  if (!obj || typeof obj !== 'object') return results;
  if (obj.type === type) results.push(obj);
  for (const v of Object.values(obj)) findNodes(v, type, results);
  return results;
}

let found = 0;
for (const s of samples) {
  if (found >= 2) break;
  const svelte = JSON.parse(fs.readFileSync(path.join(outDir, s, 'svelte.json'), 'utf8'));
  const imports = findNodes(svelte, 'ImportDeclaration');
  if (imports.length > 0) {
    console.log('=== ' + s + ' (svelte reference) ===');
    const {type, specifiers, ...rest} = imports[0];
    console.log(JSON.stringify(rest, null, 2));
    console.log();

    // Also check ExportNamedDeclaration if exists
    const exports = findNodes(svelte, 'ExportNamedDeclaration');
    if (exports.length > 0) {
      const {type: t2, specifiers: s2, declaration, ...rest2} = exports[0];
      console.log('ExportNamedDeclaration:', JSON.stringify(rest2, null, 2));
    }
    found++;
  }
}
