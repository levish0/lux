import fs from 'fs';
import path from 'path';

const outDir = 'crates/svelte-parser/tests/output';
const samples = fs.readdirSync(outDir).filter(s => {
  const d = path.join(outDir, s);
  return fs.existsSync(path.join(d, 'lux.json')) && fs.existsSync(path.join(d, 'svelte.json'));
});

function findNodes(obj, type, results = [], parentType = '') {
  if (!obj || typeof obj !== 'object') return results;
  if (obj.type === type) results.push({ node: obj, parent: parentType });
  for (const v of Object.values(obj)) findNodes(v, type, results, obj.type || parentType);
  return results;
}

// Find VariableDeclaration nodes that have loc in LUX but not in SVE
let found = 0;
for (const s of samples) {
  if (found >= 2) break;
  const lux = JSON.parse(fs.readFileSync(path.join(outDir, s, 'lux.json'), 'utf8'));
  const svelte = JSON.parse(fs.readFileSync(path.join(outDir, s, 'svelte.json'), 'utf8'));

  const luxVD = findNodes(lux, 'VariableDeclaration');
  const sveVD = findNodes(svelte, 'VariableDeclaration');

  // Check if LUX has loc on VD but SVE doesn't
  const luxHasLoc = luxVD.some(v => v.node.loc);
  const sveHasLoc = sveVD.some(v => v.node.loc);

  if (luxHasLoc && !sveHasLoc && luxVD.length > 0) {
    console.log('=== ' + s + ' ===');
    console.log('LUX VD[0]:', JSON.stringify({
      type: luxVD[0].node.type,
      loc: luxVD[0].node.loc,
      kind: luxVD[0].node.kind,
      parent: luxVD[0].parent
    }));
    console.log('SVE VD[0]:', sveVD.length > 0 ? JSON.stringify({
      type: sveVD[0].node.type,
      loc: sveVD[0].node.loc,
      kind: sveVD[0].node.kind,
      parent: sveVD[0].parent
    }) : 'NONE');
    console.log();
    found++;
  }
}
