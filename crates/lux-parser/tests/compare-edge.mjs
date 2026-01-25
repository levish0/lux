import fs from 'node:fs';

const svelte = JSON.parse(fs.readFileSync('ToParse/output/svelte.json'));
const lux = JSON.parse(fs.readFileSync('ToParse/output/lux.json'));

// Remove fields that are expected to differ
function clean(obj) {
  if (obj === null || typeof obj !== 'object') return obj;
  if (Array.isArray(obj)) return obj.map(clean);
  const result = {};
  for (const [k, v] of Object.entries(obj)) {
    // Skip metadata, comments, module (if null)
    if (k === 'metadata' || k === 'comments') continue;
    if (k === 'module' && v === null) continue;
    result[k] = clean(v);
  }
  return result;
}

const s = clean(svelte);
const l = clean(lux);

function compareDeep(a, b, path = '') {
  if (typeof a !== typeof b) {
    console.log(path + ': type mismatch', typeof a, 'vs', typeof b);
    return false;
  }
  if (a === null || b === null) {
    if (a !== b) console.log(path + ':', a, 'vs', b);
    return a === b;
  }
  if (typeof a !== 'object') {
    if (a !== b) console.log(path + ':', JSON.stringify(a), 'vs', JSON.stringify(b));
    return a === b;
  }
  if (Array.isArray(a) !== Array.isArray(b)) {
    console.log(path + ': array mismatch');
    return false;
  }
  if (Array.isArray(a)) {
    if (a.length !== b.length) {
      console.log(path + ': length', a.length, 'vs', b.length);
      return false;
    }
    return a.every((v, i) => compareDeep(v, b[i], path + '[' + i + ']'));
  }
  const keysA = Object.keys(a).sort();
  const keysB = Object.keys(b).sort();
  if (keysA.join() !== keysB.join()) {
    console.log(path + ': keys differ');
    console.log('  svelte:', keysA.filter(k => !keysB.includes(k)));
    console.log('  lux:', keysB.filter(k => !keysA.includes(k)));
  }
  let match = true;
  for (const k of keysA) {
    if (keysB.includes(k) && !compareDeep(a[k], b[k], path + '.' + k)) {
      match = false;
    }
  }
  return match;
}

if (compareDeep(s, l)) {
  console.log('MATCH: Both parsers produce identical output (after filtering metadata/comments)');
} else {
  console.log('MISMATCH: See differences above');
}
