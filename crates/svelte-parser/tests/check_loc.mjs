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

// Find ConstTag nodes and check if their declaration has loc
let found = 0;
for (const s of samples) {
  if (found >= 3) break;
  const lux = JSON.parse(fs.readFileSync(path.join(outDir, s, 'lux.json'), 'utf8'));
  const constTags = findNodes(lux, 'ConstTag');
  if (constTags.length > 0) {
    const svelte = JSON.parse(fs.readFileSync(path.join(outDir, s, 'svelte.json'), 'utf8'));
    const svelteConst = findNodes(svelte, 'ConstTag');
    console.log('=== ' + s + ' ===');
    console.log('LUX ConstTag.declaration:', JSON.stringify(constTags[0].declaration, null, 2).slice(0, 500));
    console.log('SVE ConstTag.declaration:', JSON.stringify(svelteConst[0]?.declaration, null, 2).slice(0, 500));
    console.log();
    found++;
  }
}
